//! What a press and drag over an [`EditSurface`](crate::EditSurface) tell the app: where, and the
//! text position the host resolved there, so the app moves its own caret or extends its own
//! selection.

use crate::edit::clicks::Clicks;
use crate::edit::position::TextPosition;
use crate::geometry::Point;

/// A pointer event over the surface.
#[derive(Debug, Clone, PartialEq)]
pub struct EditPointer {
    /// Which part of the gesture.
    pub phase: PointerPhase,
    /// Where, in the window's logical pixels.
    pub at: Point,
    /// The text position under it, or `None` when the host could not resolve one (no host, a
    /// busy document, a point over nothing addressable).
    pub position: Option<TextPosition>,
    /// Whether the gesture extends the selection (Shift held).
    pub extend: Extend,
    /// Which click of a quick run this press is: 2 selects a word, 3 a line.
    pub clicks: Clicks,
}

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

/// Whether a press starts a new selection or extends the current one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Extend {
    /// A new caret at the position.
    #[default]
    Fresh,
    /// Keep the anchor, move the focus to the position (Shift held).
    FromAnchor,
}

/// Whether the surface has the keyboard, told as it changes so the app shows or hides its caret.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditFocus {
    /// It has the keyboard (and the IME, where the host has one).
    In,
    /// It lost it.
    Out,
}
