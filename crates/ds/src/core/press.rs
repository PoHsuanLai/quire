//! What a pressable control hands its caller: which pointer button activated it, the
//! modifiers held and where it happened. A tray icon's right-click has to reach the app as a
//! secondary press, its middle click as a middle one, and SNI's `ContextMenu(x, y)` and
//! `Activate(x, y)` want the point.

use crate::core::geometry::units::Point;
use dioxus::prelude::Modifiers;

/// Which button pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PointerButton {
    /// The primary button, or the keyboard (Enter or Space on the focused control).
    Primary,
    /// The secondary button: a right-click, which the page reads as a context-menu request.
    Secondary,
    /// The middle button.
    Middle,
}

/// One activation of a control. `PartialEq` without `Eq`: the point is fractional pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Press {
    /// Which button.
    pub button: PointerButton,
    /// The modifiers held when it happened.
    pub modifiers: Modifiers,
    /// Where, in the surface's own coordinates (the event's client point: on a shell surface,
    /// surface-local logical pixels). A keyboard activation reports the point its event
    /// carries, which on Blitz is the origin.
    pub at: Point,
}

impl Press {
    /// A primary press with no modifiers at the origin: what a keyboard activation or a test
    /// reports.
    pub fn primary() -> Self {
        Press {
            button: PointerButton::Primary,
            modifiers: Modifiers::empty(),
            at: Point::default(),
        }
    }
}
