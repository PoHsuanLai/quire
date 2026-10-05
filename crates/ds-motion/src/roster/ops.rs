//! The roster's pure operations: reconcile with the consumer's keys, start and take back exits,
//! drop a settled batch and heal, and rest. None reads a clock: a deadline is an argument.

use super::super::anim::Anim;
use super::super::presence::{Exit, Presence};
use super::model::{Heal, RosterEntry, RosterState, RowPitch, StayError, Stayed};
use ds_core::geometry::units::Px;
use ds_core::time::stamp::Stamp;

impl<K: Clone + PartialEq> RosterState<K> {
    /// `entries` in place of the rows, everything else as it was.
    fn with_entries(self, entries: Vec<RosterEntry<K>>) -> Self {
        RosterState { entries, ..self }
    }

    /// Reconcile with the consumer's current keys: new keys enter, missing keys that are not
    /// already leaving are dropped at once (they were removed without an exit). `keys` become the
    /// listed ones.
    ///
    /// Rows keep the consumer's order. A leaving row the consumer no longer lists stays
    /// right after the listed row that preceded it (or first, if none did).
    pub fn reconcile(self, keys: &[K]) -> Self {
        let mut lingering: Vec<(Option<usize>, RosterEntry<K>)> = Vec::new();
        let mut anchor = None;
        for (at, entry) in self.entries.iter().enumerate() {
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
            match self.entries.iter().position(|entry| &entry.key == key) {
                Some(at) => {
                    entries.push(self.entries[at].clone());
                    entries.extend(after(Some(at)));
                }
                None => entries.push(RosterEntry {
                    key: key.clone(),
                    presence: Presence::Entering,
                    heal: None,
                    until: None,
                }),
            }
        }
        RosterState {
            entries,
            listed: keys.to_vec(),
            ..self
        }
    }

    /// Start the exits of every key in `keys` together, as one batch (an archive, a Clear, a
    /// group collapsing): each row plays `exit` and settles at `until`. Returns the keys that
    /// started: a key the roster does not hold, or one already leaving, is left as it is. The
    /// batch is done at `until`, and [`Self::settled_batch`] then drops them together.
    pub fn leave_batch(self, keys: &[K], exit: Exit, until: Stamp) -> (Self, Vec<K>) {
        let mut started = Vec::new();
        let entries = self
            .entries
            .iter()
            .cloned()
            .map(|entry| {
                if !keys.contains(&entry.key) || matches!(entry.presence, Presence::Leaving(_)) {
                    return entry;
                }
                started.push(entry.key.clone());
                RosterEntry {
                    presence: Presence::Leaving(exit),
                    heal: None,
                    until: Some(until),
                    ..entry
                }
            })
            .collect();
        (self.with_entries(entries), started)
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
        let mut entries = self.entries.clone();
        entries[at].presence = Presence::Present;
        entries[at].until = None;
        (self.with_entries(entries), Ok(Stayed::Restored))
    }

    /// The keys whose exit has settled by `at`: leaving, with a deadline that has come.
    pub fn due_keys(&self, at: Stamp) -> Vec<K> {
        self.entries
            .iter()
            .filter(|entry| entry.until.is_some_and(|until| at >= until))
            .map(|entry| entry.key.clone())
            .collect()
    }

    /// A batch's exits have settled: drop every key in `keys` that is still leaving, at once,
    /// and heal the rows below. Each row below the first dropped one that is not itself leaving
    /// heals by the sum of `pitch` over the dropped rows above it (each leaving row's measured
    /// height), so it starts exactly where it stood and ends in its new place. A key taken back
    /// meanwhile ([`Self::stay`]) is present, not leaving, and stays.
    pub fn settled_batch(self, keys: &[K], pitch: impl Fn(&K) -> RowPitch) -> Self {
        let mut gap: Option<Px> = None;
        let mut kept = Vec::with_capacity(self.entries.len());
        for entry in self.entries.iter().cloned() {
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
        self.with_entries(kept)
    }

    /// Every entering and healing row has settled: mark them present.
    pub fn rest(self) -> Self {
        let entries = self
            .entries
            .iter()
            .cloned()
            .map(|entry| RosterEntry {
                presence: match entry.presence {
                    Presence::Entering => Presence::Present,
                    Presence::Hidden | Presence::Present | Presence::Leaving(_) => entry.presence,
                },
                heal: None,
                ..entry
            })
            .collect();
        RosterState {
            rest_due: None,
            ..self.with_entries(entries)
        }
    }

    /// The animations that must settle before [`Self::rest`]: `heal` when a row heals and
    /// `row-in` when one enters. Empty when nothing is entering or healing.
    pub fn running(&self) -> Vec<Anim> {
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
