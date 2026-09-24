//! Driving the hover-intent machine from a caller's own pointer hooks (mailo gaps 4).
//!
//! [`HoverTarget`](super::HoverTarget) wraps what it hooks in an element of its own. A caller
//! that already has the pointer (a row's `PartHooks`, a pin, a Today item) keys the card on its
//! own events instead: [`use_hover_intent`] hands it the same hub and anchor book the target
//! uses, and [`HoverDriver::over`] files the anchor and feeds the machine in one call, so the
//! 450 ms wait, the 150 ms close and the 400 ms warm window are the hub's, not re-implemented.
//! The anchor may be [`HoverAnchor::Unplaced`]: with no layout (a server render, a host that
//! cannot measure) the card still opens, at the overlay's corner, or in place with
//! `flow: Flow::Inline`.

use super::{Anchors, use_anchors};
use crate::geometry::{MountedRef, Rect};
use crate::motion::hover_intent::HoverEvent;
use crate::overlay::hover_hub::{HoverHub, HoverKey, HoverKind, use_hover_hub};
use crate::overlay::stack::LayerStack;
use dioxus::prelude::*;

/// What a hook-keyed card is placed against.
#[derive(Debug, Clone, PartialEq)]
pub enum HoverAnchor {
    /// A rect the caller already knows: from its pointer event, or its own measurement.
    Rect(Rect),
    /// A mounted element, measured a frame after the pointer comes over it.
    Element(MountedRef),
    /// Nothing to place against: the card opens at the overlay's top-left corner (or in place,
    /// inline). Any rect filed earlier for the key is dropped, so a stale one cannot win.
    Unplaced,
}

/// The hover hub and its anchor book, for a caller keying cards on its own hooks. Copy: every
/// field is a handle.
#[derive(Clone, Copy)]
pub struct HoverDriver {
    hub: HoverHub,
    anchors: Anchors,
    stack: Option<Signal<LayerStack>>,
}

impl std::fmt::Debug for HoverDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HoverDriver")
            .field("hub", &self.hub)
            .finish_non_exhaustive()
    }
}

/// The enclosing `Ds`'s hover hub, with the anchor book every card places against.
pub fn use_hover_intent() -> HoverDriver {
    HoverDriver {
        hub: use_hover_hub(),
        anchors: use_anchors(),
        stack: try_use_context::<Signal<LayerStack>>(),
    }
}

impl HoverDriver {
    /// The pointer came over `key`'s target, a card of `kind`, placed against `anchor`. While a
    /// peek, the palette or a menu is open no card opens (`S:1790`).
    pub fn over(&self, key: HoverKey, kind: HoverKind, anchor: HoverAnchor) {
        if self.stack.is_some_and(|stack| stack.peek().top().is_some()) {
            self.hub.feed(HoverEvent::OverSuppressed);
            return;
        }
        self.anchors.file(key.clone(), anchor);
        self.hub.feed(HoverEvent::Over((key, kind)));
    }

    /// The pointer left the target: the card closes after 150 ms unless it comes back or
    /// enters the card.
    pub fn out(&self) {
        self.hub.feed(HoverEvent::Out);
    }

    /// A press in the list: the card goes at once, not warm (`S:1809`).
    pub fn press(&self) {
        self.hub.feed(HoverEvent::ClickInList);
    }

    /// The hub itself: which card is open, which is leaving, whether hover is warm.
    pub fn hub(&self) -> HoverHub {
        self.hub
    }
}
