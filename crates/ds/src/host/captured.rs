//! A pointer press the host followed past the surface's own box.

use dioxus::prelude::Modifiers;
use ds_core::geometry::units::Point;

/// The part of a press-drag-release gesture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PointerPhase {
    /// The primary button went down over the surface.
    Press,
    /// The pointer moved with the button still down after a press on the surface.
    Drag,
    /// The button came up after a press on the surface.
    Release,
}

/// A pointer event the host routes to the surface that captured the pointer at a press: every
/// move and the release until the button comes up, wherever the pointer is, so a drag selection
/// keeps following it outside the surface's box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CapturedPointer {
    /// [`PointerPhase::Drag`] for a move, [`PointerPhase::Release`] for the release.
    pub phase: PointerPhase,
    /// Where, in the window's logical pixels.
    pub at: Point,
    /// The modifiers held.
    pub modifiers: Modifiers,
}
