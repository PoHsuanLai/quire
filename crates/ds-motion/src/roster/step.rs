//! The roster's transition.
//!
//! | Input | Does |
//! |---|---|
//! | `List(keys)`, unchanged | nothing |
//! | `List(keys)` | keys newly listed that are leaving stay; with `Delist`, keys no longer listed leave as one batch settling at `at + settle(exit)`; reconcile; then the rest deadline moves out to cover the entering rows |
//! | `Leave(keys)` | those keys leave as one batch settling at `at + settle(exit)` |
//! | `Stay(key)` | a leaving row is present again |
//! | `Elapsed` at t | if the rest deadline is due, entering and healing rows are present; every leaving row due at t is dropped together, the rows below heal by the measured pitches, each dropped key comes out as `Settled`, and the rest deadline covers the healing |
//!
//! The rest deadline only ever moves later (`max`), so one deadline serves every reconcile before
//! it. A drop and a rest due together run the rest first, so the heal the drop starts is not
//! cleared at once. The wake is the earliest leaving row's settle or the rest deadline.

use super::model::{LeaveBy, Measured, RosterIn, RosterOut, RosterParams, RosterState};
use crate::settle::settle;
use ds_core::machine::Machine;
use ds_core::time::stamp::Stamp;

impl<K: Clone + PartialEq + 'static> Machine for RosterState<K> {
    type In = RosterIn<K>;
    type Out = RosterOut<K>;
    type Params = RosterParams;
    /// The heights rows measured, for the heal of a drop.
    type Ctx = Measured<K>;

    fn step(
        self,
        input: RosterIn<K>,
        at: Stamp,
        params: &RosterParams,
        measured: &Measured<K>,
    ) -> (Self, Vec<RosterOut<K>>) {
        match input {
            RosterIn::List(keys) => (self.listed(keys, at, params), Vec::new()),
            RosterIn::Leave(keys) => (self.leaving(&keys, at, params), Vec::new()),
            RosterIn::Stay(key) => (self.stay(&key).0, Vec::new()),
            RosterIn::Elapsed => self.elapsed(at, params, measured),
        }
    }

    fn wake(&self) -> Option<Stamp> {
        self.entries
            .iter()
            .filter_map(|entry| entry.until)
            .chain(self.rest_due)
            .min()
    }
}

impl<K: Clone + PartialEq> RosterState<K> {
    /// The consumer's keys again.
    fn listed(self, keys: Vec<K>, at: Stamp, params: &RosterParams) -> Self {
        if self.listed == keys {
            return self;
        }
        let gone: Vec<K> = self
            .listed
            .iter()
            .filter(|key| !keys.contains(key))
            .cloned()
            .collect();
        let added: Vec<K> = keys
            .iter()
            .filter(|key| !self.listed.contains(key))
            .cloned()
            .collect();
        // Listed again while it leaves: it stays where it is (a no-op for a new key).
        let stayed = added.iter().fold(self, |state, key| state.stay(key).0);
        let started = match params.leave {
            LeaveBy::Delist => stayed.leaving(&gone, at, params),
            LeaveBy::Action => stayed,
        };
        started.reconcile(&keys).resting(at, params)
    }

    /// Start `keys`' exits as one batch.
    fn leaving(self, keys: &[K], at: Stamp, params: &RosterParams) -> Self {
        let until = at.after_span(settle(params.exit.anim(), params.motion));
        self.leave_batch(keys, params.exit, until).0
    }

    /// The rest deadline moved out to cover the entering and healing rows, if there are any: a
    /// deadline already later is kept.
    fn resting(self, at: Stamp, params: &RosterParams) -> Self {
        let Some(length) = self
            .running()
            .into_iter()
            .map(|anim| settle(anim, params.motion))
            .max()
        else {
            return self;
        };
        let due = at.after_span(length);
        RosterState {
            rest_due: Some(self.rest_due.map_or(due, |held| held.max(due))),
            ..self
        }
    }

    /// A wake: rest what has settled, then drop what has finished leaving.
    fn elapsed(
        self,
        at: Stamp,
        params: &RosterParams,
        measured: &Measured<K>,
    ) -> (Self, Vec<RosterOut<K>>) {
        let rested = match self.rest_due {
            Some(due) if at >= due => self.rest(),
            _ => self,
        };
        let due = rested.due_keys(at);
        if due.is_empty() {
            return (rested, Vec::new());
        }
        let pitch = |key: &K| measured.of(key).unwrap_or(params.pitch);
        let dropped = rested.settled_batch(&due, pitch).resting(at, params);
        (dropped, due.into_iter().map(RosterOut::Settled).collect())
    }
}
