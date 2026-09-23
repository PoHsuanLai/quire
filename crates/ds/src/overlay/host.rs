//! The overlay registry and the host that renders it at the end of `.ds`.
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

use crate::tokens::ZLayer;
use dioxus::prelude::*;

/// Which overlay an entry is, so its owner can replace or remove it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OverlayId(pub u32);

/// The overlays currently shown, provided as context by `Ds`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Overlays {
    entries: Signal<Vec<(OverlayId, ZLayer, Element)>>,
}

impl Overlays {
    /// Show `content` on `layer`, replacing whatever `id` showed before.
    pub fn show(&self, id: OverlayId, layer: ZLayer, content: Element) {
        todo!()
    }

    /// Stop showing `id`.
    pub fn hide(&self, id: OverlayId) {
        todo!()
    }
}

/// The enclosing `Ds`'s overlay registry.
pub fn use_overlays() -> Overlays {
    todo!()
}

/// Renders every registered overlay, lowest layer first. `Ds` places it last in `.ds`.
#[component]
pub fn OverlayHost() -> Element {
    todo!()
}
