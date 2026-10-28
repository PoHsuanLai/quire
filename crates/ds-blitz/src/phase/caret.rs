//! An edit surface's caret box, read from the layout the surface's own text sits in.

use super::read::border_box;
use crate::edit_geometry::{caret, selection};
use crate::edit_locate::resolve;
use crate::edit_tree::segments;
use blitz_dom::{BaseDocument, NodeId};
use ds::host::position::{TextPosition, TextRange};
use ds::prelude::{Point, Rect};

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

/// The boxes `range` covers in `surface`, relative to the surface's border box; empty when the
/// surface or either end of the range is not in the layout.
pub(super) fn selection_boxes(doc: &BaseDocument, surface: NodeId, range: &TextRange) -> Vec<Rect> {
    let Some(bounds) = border_box(doc, surface).filter(|_| range.anchor != range.focus) else {
        return Vec::new();
    };
    let (Some(from), Some(to)) = (
        resolve(doc, surface, &range.anchor),
        resolve(doc, surface, &range.focus),
    ) else {
        return Vec::new();
    };
    selection(doc, &segments(doc, surface), from, to)
        .unwrap_or_default()
        .into_iter()
        .map(|found| Rect {
            origin: Point {
                x: found.origin.x - bounds.origin.x,
                y: found.origin.y - bounds.origin.y,
            },
            size: found.size,
        })
        .collect()
}
