//! The list roster as a pure state machine: which keys are on screen and in what state, so a
//! leaving row stays in the tree until its exit settles and the rows below heal
//! (design/05-MOTION.md principle 10, design/06-INTERACTIONS.md "row lifecycle").
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

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
        todo!()
    }

    /// The rows to draw, leaving ones included.
    pub fn entries(&self) -> &[RosterEntry<K>] {
        &self.entries
    }

    /// Reconcile with the consumer's current keys: new keys enter, missing keys that are not
    /// already leaving are dropped at once (they were removed without an exit).
    pub fn reconcile(self, keys: &[K]) -> Self {
        todo!()
    }

    /// Start `key`'s exit. It stays in [`Self::entries`] until [`Self::settled`]; the
    /// returned animation is the one to settle (an unread row's fold is `FoldHeavy`).
    pub fn leave(self, key: &K, exit: Exit, emphasis: Emphasis) -> (Self, Anim) {
        todo!()
    }

    /// `key`'s exit has settled: drop it and start the rows below healing.
    pub fn settled(self, key: &K) -> Self {
        todo!()
    }

    /// Every entering and healing row has settled: mark them present.
    pub fn rest(self) -> Self {
        todo!()
    }
}
