//! The list roster as a pure state machine: which keys are on screen and in what state, so a
//! leaving row stays in the tree until its exit settles and the rows below close the gap
//! (design/30 section 1.3: insert fades and slides down over `--t-move`, removal fades and
//! slides up over `--t-quick`, the rows below close the gap over `--t-move`).

#[cfg(feature = "dioxus")]
use super::anim::Anim;
use super::presence::{Exit, Presence};
use ds_core::geometry::units::Px;

/// A row's height plus the gap below it: how far the rows below move when it goes.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct RowPitch(pub Px);

/// A row sliding up into the gap a removed row left: `data-presence="healing"`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Heal {
    /// How far it starts below its resting place.
    pub dy: Px,
}

/// The `data-presence` word of a row: `healing` while it heals, else its presence's.
pub fn presence_slug(presence: Presence, heal: Option<Heal>) -> &'static str {
    match heal {
        Some(_) => "healing",
        None => presence.slug(),
    }
}

/// One row the roster is drawing.
#[derive(Debug, Clone, PartialEq)]
pub struct RosterEntry<K> {
    /// The consumer's key.
    pub key: K,
    /// Its motion state.
    pub presence: Presence,
    /// Its heal, while it slides into a gap; only a present row heals.
    pub heal: Option<Heal>,
}

/// Every row on screen, in order, with its motion state.
#[derive(Debug, Clone, PartialEq)]
pub struct RosterState<K> {
    entries: Vec<RosterEntry<K>>,
}

impl<K: Clone + PartialEq> RosterState<K> {
    /// A roster showing `keys` for the first time: every row simply there (nothing sweeps in on
    /// first show).
    pub fn first_show(keys: &[K]) -> Self {
        let entries = keys
            .iter()
            .map(|key| RosterEntry {
                key: key.clone(),
                presence: Presence::Present,
                heal: None,
            })
            .collect();
        RosterState { entries }
    }

    /// The rows to draw, leaving ones included.
    pub fn entries(&self) -> &[RosterEntry<K>] {
        &self.entries
    }

    /// Reconcile with the consumer's current keys: new keys enter, missing keys that are not
    /// already leaving are dropped at once (they were removed without an exit).
    ///
    /// Rows keep the consumer's order. A leaving row the consumer no longer lists stays
    /// right after the listed row that preceded it (or first, if none did).
    pub fn reconcile(self, keys: &[K]) -> Self {
        let old = self.entries;
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
        for key in keys {
            match old.iter().position(|entry| &entry.key == key) {
                Some(at) => {
                    entries.push(old[at].clone());
                    entries.extend(after(Some(at)));
                }
                None => entries.push(RosterEntry {
                    key: key.clone(),
                    presence: Presence::Entering,
                    heal: None,
                }),
            }
        }
        RosterState { entries }
    }

    /// Start the exits of every key in `keys` together, as one batch (an archive, a Clear, a
    /// group collapsing): each row plays `exit`. Returns the keys that started: a key the roster
    /// does not hold, or one already leaving, is left as it is. The batch is done at the exit's
    /// settle, and [`Self::settled_batch`] then drops them together.
    pub fn leave_batch(self, keys: &[K], exit: Exit) -> (Self, Vec<K>) {
        let mut started = Vec::new();
        let entries = self
            .entries
            .into_iter()
            .map(|entry| {
                if !keys.contains(&entry.key) || matches!(entry.presence, Presence::Leaving(_)) {
                    return entry;
                }
                started.push(entry.key.clone());
                RosterEntry {
                    presence: Presence::Leaving(exit),
                    heal: None,
                    ..entry
                }
            })
            .collect();
        (RosterState { entries }, started)
    }

    /// Take a leaving row's exit back: it is present again, in place, with nothing below it
    /// healing (an undo during the exit, before the row was dropped). A row that is not leaving
    /// is left as it is; a key the roster does not hold is [`StayError::UnknownKey`], because a
    /// row already dropped cannot stay: the consumer lists it again and it enters.
    pub fn stay(self, key: &K) -> (Self, Result<Stayed, StayError>) {
        let Some(at) = self.entries.iter().position(|entry| &entry.key == key) else {
            return (self, Err(StayError::UnknownKey));
        };
        if !matches!(self.entries[at].presence, Presence::Leaving(_)) {
            return (self, Ok(Stayed::Unchanged));
        }
        let RosterState { mut entries } = self;
        entries[at].presence = Presence::Present;
        (RosterState { entries }, Ok(Stayed::Restored))
    }

    /// A batch's exits have settled: drop every key in `keys` that is still leaving, at once,
    /// and heal the rows below. Each row below the first dropped one that is not itself leaving
    /// heals by the sum of `pitch` over the dropped rows above it (each leaving row's measured
    /// height), so it starts exactly where it stood and ends in its new place. A key taken back
    /// meanwhile ([`Self::stay`]) is present, not leaving, and stays.
    pub fn settled_batch(self, keys: &[K], pitch: impl Fn(&K) -> RowPitch) -> Self {
        let mut gap: Option<Px> = None;
        let mut kept = Vec::with_capacity(self.entries.len());
        for entry in self.entries {
            let leaving = matches!(entry.presence, Presence::Leaving(_));
            if leaving && keys.contains(&entry.key) {
                gap = Some(gap.unwrap_or(Px(0.0)) + pitch(&entry.key).0);
                continue;
            }
            match gap {
                Some(dy) if !leaving => kept.push(RosterEntry {
                    presence: Presence::Present,
                    heal: Some(Heal { dy }),
                    ..entry
                }),
                Some(_) | None => kept.push(entry),
            }
        }
        RosterState { entries: kept }
    }

    /// Every entering and healing row has settled: mark them present.
    pub fn rest(self) -> Self {
        let entries = self
            .entries
            .into_iter()
            .map(|entry| RosterEntry {
                presence: match entry.presence {
                    Presence::Entering => Presence::Present,
                    Presence::Hidden | Presence::Present | Presence::Leaving(_) => entry.presence,
                },
                heal: None,
                ..entry
            })
            .collect();
        RosterState { entries }
    }

    /// The animations that must settle before [`Self::rest`]: `heal` when a row heals and
    /// `row-in` when one enters. Empty when nothing is entering or healing.
    #[cfg(feature = "dioxus")]
    pub(crate) fn running(&self) -> Vec<Anim> {
        let heal = self.entries.iter().any(|entry| entry.heal.is_some());
        let enter = self
            .entries
            .iter()
            .any(|entry| entry.presence == Presence::Entering);
        [(heal, Anim::Heal), (enter, Anim::RowIn)]
            .into_iter()
            .filter_map(|(running, anim)| running.then_some(anim))
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
