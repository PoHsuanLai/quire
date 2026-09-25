//! Which link in which frame is under a point of the app's document. Blitz forwards a pointer
//! move into a frame's document itself, but tells the app nothing about it; ds-native hit-tests
//! the same way (the frame's offset in its parent, its own scroll) so it can say when the
//! pointer comes onto and leaves a link, for the app's link pill.

use crate::frame_anchor::{LinkFacts, anchor_at};
use crate::origin::FrameId;
use blitz_dom::{BaseDocument, NodeId};
use ds::Point;

/// A link under the pointer, inside a frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LinkUnder {
    /// The frame's document.
    pub(crate) frame: FrameId,
    /// The anchor element in it: the same link while this stays the same.
    pub(crate) anchor: NodeId,
    /// What the link says.
    pub(crate) facts: LinkFacts,
}

/// The link under `at` (the app document's client coordinates) inside a frame, if any. A link
/// in the app's own document is the app's to track.
pub(crate) fn link_under(top: &BaseDocument, at: Point) -> Option<LinkUnder> {
    let scroll = top.viewport_scroll();
    let page = (at.x.0 + scroll.x as f32, at.y.0 + scroll.y as f32);
    descend(top, page, None)
}

/// Hit-test `doc` at page point `(x, y)`, going into the frame hit if there is one; `frame` is
/// the frame `doc` is, or `None` for the app's document.
fn descend(doc: &BaseDocument, (x, y): (f32, f32), frame: Option<FrameId>) -> Option<LinkUnder> {
    let hit = doc.hit(x, y)?;
    let node = doc.get_node(hit.node_id)?;
    match node.subdoc() {
        Some(sub) => {
            let offset = node.absolute_position(0.0, 0.0);
            let inner = sub.inner();
            let scroll = inner.viewport_scroll();
            let inside = (
                x - offset.x + scroll.x as f32,
                y - offset.y + scroll.y as f32,
            );
            descend(&inner, inside, Some(FrameId::of(sub.id())))
        }
        None => {
            let (anchor, facts) = anchor_at(doc, hit.node_id)?;
            Some(LinkUnder {
                frame: frame?,
                anchor,
                facts,
            })
        }
    }
}
