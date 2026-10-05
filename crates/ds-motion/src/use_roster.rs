//! The roster as one hook: [`RosterState`] run by [`use_machine`]. Its settle deadlines are the
//! machine's wake, so the hook owns no timer; it passes the consumer's keys in on every render
//! and reads the rows back.
//!
//! Rows leave in batches: every key that starts leaving in one go plays the exit together, and
//! when the exit has settled they are dropped at once and the rows below heal by the heights the
//! dropped rows measured. A key listed again while it leaves stays where it is.

use super::machine::{MachineRef, use_machine};
use super::presence::Exit;
use super::roster::{
    Measured, RosterEntry, RosterIn, RosterOut, RosterParams, RosterState, RowPitch, StayError,
    Stayed,
};
use dioxus::prelude::*;
use ds_style::scope::use_scope_signal;

pub use super::roster::LeaveBy;

/// What a roster does with its rows.
#[derive(Debug)]
pub struct RosterSpec<K: 'static> {
    /// How a row leaves.
    pub leave: LeaveBy,
    /// The exit its rows play, read when a batch starts.
    pub exit: Exit,
    /// How far the rows below heal for a row that measured nothing.
    pub pitch: RowPitch,
    /// Hears each dropped key once its batch has settled.
    pub on_settled: Option<EventHandler<K>>,
}

impl<K: 'static> Clone for RosterSpec<K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K: 'static> Copy for RosterSpec<K> {}

/// The heights rows measured, by key: how far the rows below heal when one leaves.
#[derive(Debug, PartialEq)]
pub struct Pitches<K: 'static>(CopyValue<Vec<(K, RowPitch)>>);

impl<K: 'static> Clone for Pitches<K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K: 'static> Copy for Pitches<K> {}

impl<K: Clone + PartialEq + 'static> Pitches<K> {
    /// Record `key`'s pitch.
    pub fn set(&self, key: K, pitch: RowPitch) {
        let mut book = self.0;
        let _ = book.try_write().map(|mut book| {
            book.retain(|(held, _)| *held != key);
            book.push((key, pitch));
        });
    }

    fn forget(&self, key: &K) {
        let mut book = self.0;
        let _ = book
            .try_write()
            .map(|mut book| book.retain(|(held, _)| held != key));
    }
}

/// A live roster: read its entries in render, start exits from handlers.
#[derive(Debug)]
pub struct Roster<K: Clone + PartialEq + 'static> {
    machine: MachineRef<RosterState<K>>,
    pitches: Pitches<K>,
}

impl<K: Clone + PartialEq + 'static> Clone for Roster<K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K: Clone + PartialEq + 'static> Copy for Roster<K> {}

impl<K: Clone + PartialEq + 'static> Roster<K> {
    /// The rows to draw, leaving and healing ones included.
    pub fn entries(&self) -> Vec<RosterEntry<K>> {
        self.machine.state().read().entries().to_vec()
    }

    /// The book rows record their measured heights in, for the rows below to heal by.
    pub fn pitches(&self) -> Pitches<K> {
        self.pitches
    }

    /// Start `key`'s exit; it is dropped, and the rows below heal, when the exit settles.
    pub fn leave(&self, key: K) {
        self.machine.send(RosterIn::Leave(vec![key]));
    }

    /// Take `key`'s exit back while it plays: the row is present again where it was and is not
    /// dropped when its batch settles, so nothing below it heals (an undo before the row was
    /// dropped). The consumer lists the key again in the same handler, so the next reconcile
    /// keeps the row. A row that is not leaving is unchanged; a key the roster no longer holds
    /// is [`StayError::UnknownKey`] (its exit settled: list it again and it enters).
    pub fn stay(&self, key: K) -> Result<Stayed, StayError> {
        let held = self
            .machine
            .state()
            .try_peek()
            .map(|state| state.clone())
            .map_err(|_| StayError::Unmounted)?;
        let (_, stayed) = held.stay(&key);
        self.machine.send(RosterIn::Stay(key));
        stayed
    }
}

/// A roster over `keys`, which the consumer passes on every render. The first render shows
/// every key present; after that a change in `keys` reconciles: new keys enter, and a key that
/// goes missing leaves as `spec.leave` says (keys missing at once are one batch). The reconcile
/// happens in the render, so the rows to draw are right in the render that lists them; the
/// settle timers are the machine's wake, started after the render.
pub fn use_roster<K: Clone + PartialEq + 'static>(keys: Vec<K>, spec: RosterSpec<K>) -> Roster<K> {
    let scope = use_scope_signal();
    let pitches = Pitches(use_hook(|| CopyValue::new(Vec::new())));
    let params = RosterParams {
        leave: spec.leave,
        exit: spec.exit,
        pitch: spec.pitch,
        motion: scope.peek().resolved.motion,
    };
    let on_settled = spec.on_settled;
    let machine = use_machine(
        |_| RosterState::first_show(&keys),
        params,
        move || {
            Measured(
                pitches
                    .0
                    .try_peek()
                    .map(|book| book.clone())
                    .unwrap_or_default(),
            )
        },
        move |RosterOut::Settled(key), _| {
            pitches.forget(&key);
            if let Some(on_settled) = on_settled {
                on_settled.call(key);
            }
        },
    );
    machine.send_from_render(RosterIn::List(keys));
    Roster { machine, pitches }
}
