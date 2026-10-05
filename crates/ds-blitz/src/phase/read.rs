//! What the phase reads from a laid-out document and the one write it makes, as functions over the
//! document.

use blitz_dom::{BaseDocument, NodeId, ScrollBehavior};
use ds::host::phase::PhaseWrite;
use ds::prelude::{Point, Px, Rect, Size};

/// `node`'s border box in logical pixels; `None` when the node is gone.
pub(crate) fn border_box(doc: &BaseDocument, node: NodeId) -> Option<Rect> {
    let found = doc.get_client_bounding_rect(node)?;
    Some(Rect {
        origin: Point {
            x: Px(found.x as f32),
            y: Px(found.y as f32),
        },
        size: Size {
            width: Px(found.width as f32),
            height: Px(found.height as f32),
        },
    })
}

/// Apply `write` to `node`; whether it moved anything.
pub(super) fn apply(doc: &mut BaseDocument, node: NodeId, write: PhaseWrite) -> Moved {
    match write {
        PhaseWrite::ScrollTo(offset) => {
            let Some(before) = doc.get_node(node).map(|found| *found.scroll_offset()) else {
                return Moved::No;
            };
            doc.scroll_to(node, before.x, f64::from(offset.0), ScrollBehavior::Instant);
            let after = doc.get_node(node).map(|found| *found.scroll_offset());
            match after == Some(before) {
                true => Moved::No,
                false => Moved::Yes,
            }
        }
    }
}

/// Whether a write changed the document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Moved {
    Yes,
    No,
}
