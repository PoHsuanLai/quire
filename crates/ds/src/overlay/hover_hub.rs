//! The one hover manager every card goes through: it owns the [`crate::HoverIntent`] machine
//! and its timers, and stamps `data-hover="warm|cold"` on `.ds` (design/04-COMPONENTS.md
//! O-11).
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

use crate::motion::hover_intent::{HoverEvent, HoverIntent};
use dioxus::prelude::*;

/// What a hover target is, by the consumer's own key: `sender:3`, `thread:88`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HoverKey(pub String);

/// Which card a target opens (design/04-COMPONENTS.md section 22).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HoverKind {
    /// A list row: the thread card, placed right of the list column.
    Thread,
    /// A name: the sender card, placed below.
    Sender,
    /// An account tile.
    Account,
    /// A sidebar entry (pin, Today): the narrow side card, placed right of the target.
    Side,
}

/// Whether cards and fly labels open at once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HoverWarmth {
    /// A card is open or closed less than HoverWarm ago: no wait.
    Warm,
    /// Wait for intent.
    #[default]
    Cold,
}

/// The hover manager, provided as context by `Ds`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HoverHub {
    intent: Signal<HoverIntent<(HoverKey, HoverKind)>>,
}

impl HoverHub {
    /// Feed an event; the hub starts and cancels its own timers.
    pub fn feed(&self, event: HoverEvent<(HoverKey, HoverKind)>) {
        todo!()
    }

    /// The card that is open (or closing), for the consumer to render.
    pub fn open(&self) -> Option<(HoverKey, HoverKind)> {
        todo!()
    }

    /// Whether cards open at once right now.
    pub fn warmth(&self) -> HoverWarmth {
        todo!()
    }
}

/// The enclosing `Ds`'s hover manager.
pub fn use_hover_hub() -> HoverHub {
    todo!()
}
