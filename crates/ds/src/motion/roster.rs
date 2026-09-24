//! The list roster as a pure state machine: which keys are on screen and in what state, so a
//! leaving row stays in the tree until its exit settles and the rows below heal
//! (design/05-MOTION.md principle 10, design/06-INTERACTIONS.md "row lifecycle").

use super::anim::Anim;
use super::presence::{Exit, Presence};
use crate::components::vocab::{Emphasis, StaggerIndex};
use crate::geometry::units::Px;

/// A row's height plus the gap below it: how far the rows below heal (`dy = height + 5`).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct RowPitch(pub Px);

/// One row the roster is drawing.
#[derive(Debug, Clone, PartialEq)]
pub struct RosterEntry<K> {
    /// The consumer's key.
    pub key: K,
    /// Its motion state.
    pub presence: Presence,
    /// Its entrance stagger.
    pub index: StaggerIndex,
}

/// Every row on screen, in order, with its motion state.
#[derive(Debug, Clone, PartialEq)]
pub struct RosterState<K> {
    entries: Vec<RosterEntry<K>>,
    pitch: RowPitch,
}

impl<K: Clone + PartialEq> RosterState<K> {
    /// A roster showing `keys` for the first time: every row entering, staggered.
    pub fn first_show(keys: &[K], pitch: RowPitch) -> Self {
        let entries = keys
            .iter()
            .enumerate()
            .map(|(n, key)| RosterEntry {
                key: key.clone(),
                presence: Presence::Entering,
                index: StaggerIndex::new(n),
            })
            .collect();
        RosterState { entries, pitch }
    }

    /// The rows to draw, leaving ones included.
    pub fn entries(&self) -> &[RosterEntry<K>] {
        &self.entries
    }

    /// Reconcile with the consumer's current keys: new keys enter, missing keys that are not
    /// already leaving are dropped at once (they were removed without an exit).
    ///
    /// Rows keep the consumer's order. A leaving row the consumer no longer lists stays
    /// right after the listed row that preceded it (or first, if none did). New keys enter,
    /// staggered among themselves from 0.
    pub fn reconcile(self, keys: &[K]) -> Self {
        let RosterState {
            entries: old,
            pitch,
        } = self;
        let mut lingering: Vec<(Option<usize>, RosterEntry<K>)> = Vec::new();
        let mut anchor = None;
        for (at, entry) in old.iter().enumerate() {
            if keys.contains(&entry.key) {
                anchor = Some(at);
            } else if matches!(entry.presence, Presence::Leaving(_)) {
                lingering.push((anchor, entry.clone()));
            }
        }
        let after = |anchor: Option<usize>| {
            lingering
                .iter()
                .filter(move |(a, _)| *a == anchor)
                .map(|(_, entry)| entry.clone())
        };
        let mut entries: Vec<RosterEntry<K>> = after(None).collect();
        let mut arrivals = 0;
        for key in keys {
            match old.iter().position(|entry| &entry.key == key) {
                Some(at) => {
                    entries.push(old[at].clone());
                    entries.extend(after(Some(at)));
                }
                None => {
                    entries.push(RosterEntry {
                        key: key.clone(),
                        presence: Presence::Entering,
                        index: StaggerIndex::new(arrivals),
                    });
                    arrivals += 1;
                }
            }
        }
        RosterState { entries, pitch }
    }

    /// Start `key`'s exit. It stays in [`Self::entries`] until [`Self::settled`]; the
    /// returned animation is the one to settle (an unread row's fold is `FoldHeavy`).
    pub fn leave(self, key: &K, exit: Exit, emphasis: Emphasis) -> (Self, Anim) {
        let entries = self
            .entries
            .into_iter()
            .map(|entry| {
                if &entry.key == key {
                    RosterEntry {
                        presence: Presence::Leaving(exit),
                        ..entry
                    }
                } else {
                    entry
                }
            })
            .collect();
        (
            RosterState {
                entries,
                pitch: self.pitch,
            },
            exit_anim(exit, emphasis),
        )
    }

    /// Take `key`'s exit back: a leaving row is present again, in place, with nothing below it
    /// healing (an undo during the exit, before the row was dropped). A row that is not leaving
    /// is left as it is; a key the roster does not hold is [`StayError::UnknownKey`], because
    /// a row already dropped cannot stay: the consumer lists it again and it enters.
    ///
    /// Only [`Self::settled`] starts a heal, so a stay before it leaves the rows below as they
    /// were; the hook cancels the exit's settle timer so it never runs for the stayed row.
    pub fn stay(self, key: &K) -> (Self, Result<Stayed, StayError>) {
        let Some(at) = self.entries.iter().position(|entry| &entry.key == key) else {
            return (self, Err(StayError::UnknownKey));
        };
        if !matches!(self.entries[at].presence, Presence::Leaving(_)) {
            return (self, Ok(Stayed::Unchanged));
        }
        let RosterState { mut entries, pitch } = self;
        entries[at].presence = Presence::Present;
        (RosterState { entries, pitch }, Ok(Stayed::Restored))
    }

    /// `key`'s exit has settled: drop it and start the rows below healing.
    ///
    /// Every row below it that is not itself leaving starts `heal` from one pitch down; the
    /// first row below has heal index 0, the next 1, and so on (capped like the stagger).
    /// Nothing happens for a key that is not leaving.
    pub fn settled(self, key: &K) -> Self {
        let RosterState { entries, pitch } = self;
        let Some(at) = entries
            .iter()
            .position(|e| &e.key == key && matches!(e.presence, Presence::Leaving(_)))
        else {
            return RosterState { entries, pitch };
        };
        let mut below = 0;
        let entries = entries
            .into_iter()
            .enumerate()
            .filter(|&(n, _)| n != at)
            .map(|(n, entry)| {
                if n < at || matches!(entry.presence, Presence::Leaving(_)) {
                    return entry;
                }
                let d = StaggerIndex::new(below);
                below += 1;
                RosterEntry {
                    presence: Presence::Healing { dy: pitch.0, d },
                    ..entry
                }
            })
            .collect();
        RosterState { entries, pitch }
    }

    /// Every entering and healing row has settled: mark them present.
    pub fn rest(self) -> Self {
        let entries = self
            .entries
            .into_iter()
            .map(|entry| match entry.presence {
                Presence::Entering | Presence::Healing { .. } => RosterEntry {
                    presence: Presence::Present,
                    ..entry
                },
                Presence::Present | Presence::Leaving(_) => entry,
            })
            .collect();
        RosterState {
            entries,
            pitch: self.pitch,
        }
    }

    /// The animations that must settle before [`Self::rest`]: `heal` at the largest heal
    /// index, and both entrances (`rise` on first show, `row-in` for an arrival) at the
    /// largest stagger index. Empty when nothing is entering or healing.
    pub(crate) fn running(&self) -> Vec<(Anim, StaggerIndex)> {
        let heal = self
            .entries
            .iter()
            .filter_map(|e| match e.presence {
                Presence::Healing { d, .. } => Some(d),
                _ => None,
            })
            .max()
            .map(|d| (Anim::Heal, d));
        let enter = self
            .entries
            .iter()
            .filter(|e| e.presence == Presence::Entering)
            .map(|e| e.index)
            .max();
        heal.into_iter()
            .chain(
                enter
                    .into_iter()
                    .flat_map(|i| [(Anim::Rise, i), (Anim::RowIn, i)]),
            )
            .collect()
    }
}

/// What a [`RosterState::stay`] did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stayed {
    /// The row was leaving and is present again.
    Restored,
    /// The row was not leaving (entering, present or healing); nothing changed.
    Unchanged,
}

/// Why a stay could not happen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StayError {
    /// The roster holds no row with that key: never listed, or already dropped after its exit.
    UnknownKey,
    /// The roster's owner is gone (the list unmounted); only the hook reports this.
    Unmounted,
}

/// The animation an exit plays: an unread (`Emphasis::Strong`) row plays the heavy variant of
/// every row exit, 15 % slower (design/05-MOTION.md principle 6, section 8). A Today entry's
/// `tab-out` has no heavy variant.
pub(crate) fn exit_anim(exit: Exit, emphasis: Emphasis) -> Anim {
    match (exit, emphasis) {
        (Exit::Fold, Emphasis::Strong) => Anim::FoldHeavy,
        (Exit::Fold, Emphasis::Plain) => Anim::Fold,
        (Exit::Curl, Emphasis::Strong) => Anim::CurlHeavy,
        (Exit::Curl, Emphasis::Plain) => Anim::Curl,
        (Exit::Crumple, Emphasis::Strong) => Anim::CrumpleHeavy,
        (Exit::Crumple, Emphasis::Plain) => Anim::Crumple,
        (Exit::TabOut, _) => Anim::TabOut,
    }
}
