//! Popover: the shared floating surface under menus, hover cards, tooltips, the bubble and the
//! palette. Placement in Rust; Esc and outside click through the layer stack
//! (design/04-COMPONENTS.md section 21).

use crate::geometry::{Anchor, Placement, Px};
use dioxus::prelude::*;

/// Which surface a popover draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Elevation {
    /// Menus, hover cards: `--shadow-pop`.
    #[default]
    Pop,
    /// The selection bubble.
    Bubble,
    /// The palette, peek: `--shadow-sheet`.
    Sheet,
}

/// What closes a popover.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Dismiss {
    /// Escape, or a click outside.
    #[default]
    EscAndOutside,
    /// Escape only.
    EscOnly,
    /// Only its owner.
    None,
}

/// A floating surface.
#[component]
pub fn Popover(
    anchor: Anchor,
    placement: Placement,
    gap: Px,
    #[props(default)] elevation: Elevation,
    #[props(default)] dismiss: Dismiss,
    onclose: EventHandler<()>,
    children: Element,
) -> Element {
    todo!()
}
