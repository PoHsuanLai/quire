//! The hover-card intent machine: wait for intent, stay warm, never mark read, never fetch
//! (design/06-INTERACTIONS.md section 3). Pure: an event and the time in, the next state and
//! one effect out.
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

use std::time::{Duration, Instant};

/// Where the machine is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentPhase<K> {
    /// No card, nothing pending.
    Idle,
    /// The pointer rests on `key`; its card opens at `due`.
    Pending {
        /// The target under the pointer.
        key: K,
        /// When the open timer fires.
        due: Instant,
    },
    /// `key`'s card is open.
    Open {
        /// The open card's key.
        key: K,
    },
    /// The pointer left `key`'s target and card; it closes at `due`.
    Closing {
        /// The closing card's key.
        key: K,
        /// When the close timer fires.
        due: Instant,
    },
}

/// Something that happened to the hover machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HoverEvent<K> {
    /// The pointer came over the innermost target `key`.
    Over(K),
    /// The pointer came over a target while cards are suppressed (a leaving row, an open peek
    /// or palette).
    OverSuppressed,
    /// The pointer left the target for something that is neither the card nor the same target.
    Out,
    /// The pointer entered the open card.
    EnterCard,
    /// The pointer left the open card.
    LeaveCard,
    /// The open timer fired.
    OpenDue,
    /// The close timer fired.
    CloseDue,
    /// A click landed in the list (capture phase).
    ClickInList,
    /// Space was pressed with focus outside a text field.
    SpaceKey,
}

/// What the caller must do after a step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentEffect<K> {
    /// Nothing.
    None,
    /// Cancel any open timer and start one that fires after `after` (0 when warm).
    StartOpen {
        /// HoverOpen, or zero when warm.
        after: Duration,
    },
    /// Cancel the open timer.
    CancelOpen,
    /// Start the close timer (HoverClose).
    StartClose {
        /// HoverClose.
        after: Duration,
    },
    /// Cancel the close timer.
    CancelClose,
    /// Remove any open card instantly and open `key`'s card (`hc-in`).
    Open(K),
    /// Play `hc-out` on `key`'s card and unmount it at `settle(HcOut)`; the hub is warm.
    Close(K),
    /// Remove `key`'s card instantly, no exit, not warm.
    Remove(K),
    /// Close `key`'s card (warm) and open the peek for it.
    Peek(K),
}

/// The machine: a phase plus an independent warm deadline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoverIntent<K> {
    phase: IntentPhase<K>,
    warm_until: Option<Instant>,
}

impl<K> Default for HoverIntent<K> {
    fn default() -> Self {
        HoverIntent {
            phase: IntentPhase::Idle,
            warm_until: None,
        }
    }
}

impl<K: Clone + PartialEq> HoverIntent<K> {
    /// Apply `event` at `now` (450 ms open, 150 ms close, 400 ms warm).
    pub fn step(self, event: HoverEvent<K>, now: Instant) -> (Self, IntentEffect<K>) {
        todo!()
    }

    /// Where the machine is.
    pub fn phase(&self) -> &IntentPhase<K> {
        &self.phase
    }

    /// Whether cards open at once: a card is open, or one closed less than HoverWarm ago.
    pub fn warmth(&self, now: Instant) -> crate::overlay::hover_hub::HoverWarmth {
        todo!()
    }
}
