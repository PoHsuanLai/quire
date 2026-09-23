//! The roster as a hook: [`RosterState`] in a signal, with the settle timers started for it.
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

use super::presence::Exit;
use super::roster::{RosterEntry, RosterState, RowPitch};
use crate::components::vocab::Emphasis;
use dioxus::prelude::*;

/// A live roster: read its entries in render, start exits from handlers.
#[derive(Debug, PartialEq)]
pub struct Roster<K: 'static> {
    state: Signal<RosterState<K>>,
}

impl<K: 'static> Clone for Roster<K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K: 'static> Copy for Roster<K> {}

impl<K: Clone + PartialEq + 'static> Roster<K> {
    /// The rows to draw, leaving and healing ones included.
    pub fn entries(&self) -> Vec<RosterEntry<K>> {
        todo!()
    }

    /// Start `key`'s exit; it is dropped, and the rows below heal, when the exit settles. An
    /// unread (`Emphasis::Strong`) row exits at `--t-big-heavy`.
    pub fn leave(&self, key: K, exit: Exit, emphasis: Emphasis) {
        todo!()
    }
}

/// A roster over `keys`, which the consumer passes on every render. Stagger is capped at 12.
pub fn use_roster<K: Clone + PartialEq + 'static>(keys: Vec<K>, pitch: RowPitch) -> Roster<K> {
    todo!()
}
