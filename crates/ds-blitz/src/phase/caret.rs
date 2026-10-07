//! An edit surface's caret box, read from the layout the surface's own text sits in, and whether
//! the phase should read it before the frame paints.

use super::read::border_box;
use crate::edit_geometry::caret;
use crate::edit_locate::resolve;
use blitz_dom::{BaseDocument, NodeId};
use ds::host::position::TextPosition;
use ds::prelude::{Point, Rect};

/// Whether the phase has work to do before a frame paints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Early {
    /// A caret moved since its box was read: lay out, publish, and render what that changed, so
    /// the frame draws the caret where the text it follows is.
    Wanted,
    /// Nothing moved; the frame paints as it is.
    Idle,
}

/// The caret's box at `at` in `surface`, relative to the surface's border box; `None` when the
/// surface or the position is not in the layout.
pub(super) fn caret_box(doc: &BaseDocument, surface: NodeId, at: &TextPosition) -> Option<Rect> {
    let found = caret(doc, resolve(doc, surface, at)?)?;
    let bounds = border_box(doc, surface)?;
    Some(Rect {
        origin: Point {
            x: found.origin.x - bounds.origin.x,
            y: found.origin.y - bounds.origin.y,
        },
        size: found.size,
    })
}
