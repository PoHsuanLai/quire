//! What a press and drag over an `EditSurface` tell the app: where, and the
//! text position the host resolved there, so the app moves its own caret or extends its own
//! selection.

use crate::edit::clicks::Clicks;
use crate::host::captured::PointerPhase;
use crate::host::position::TextPosition;
use ds_core::geometry::units::Point;

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

impl EditPointer {
    /// A `phase` at `at` the host resolved to no text position: a fresh selection, the first
    /// click of a run. The builders below set the rest, so a consumer's test writes the gesture
    /// it means.
    pub fn new(phase: PointerPhase, at: Point) -> Self {
        EditPointer {
            phase,
            at,
            position: None,
            extend: Extend::Fresh,
            clicks: Clicks(1),
        }
    }

    /// The same gesture over `position`.
    pub fn over(self, position: TextPosition) -> Self {
        EditPointer {
            position: Some(position),
            ..self
        }
    }

    /// The same gesture with Shift held: it extends the selection.
    pub fn extending(self) -> Self {
        EditPointer {
            extend: Extend::FromAnchor,
            ..self
        }
    }

    /// The same gesture as press number `clicks` of a quick run.
    pub fn clicking(self, clicks: Clicks) -> Self {
        EditPointer { clicks, ..self }
    }
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

#[cfg(test)]
mod tests {
    use super::{EditPointer, Extend};
    use crate::edit::clicks::Clicks;
    use crate::host::captured::PointerPhase;
    use crate::host::position::TextPosition;
    use ds_core::geometry::units::{Point, Px};

    #[test]
    fn a_pointer_is_built_from_the_gesture_it_means() {
        let at = Point {
            x: Px(4.0),
            y: Px(9.0),
        };
        let press = EditPointer::new(PointerPhase::Press, at);
        assert_eq!(press.position, None);
        assert_eq!(press.extend, Extend::Fresh);
        assert_eq!(press.clicks, Clicks(1));
        let word = press
            .over(TextPosition::new("0", 3))
            .extending()
            .clicking(Clicks(2));
        assert_eq!(word.position, Some(TextPosition::new("0", 3)));
        assert_eq!(word.extend, Extend::FromAnchor);
        assert_eq!(word.clicks, Clicks(2));
        assert_eq!((word.phase, word.at), (PointerPhase::Press, at));
    }
}
