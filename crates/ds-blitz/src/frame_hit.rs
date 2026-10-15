//! Which link, or which image, in which frame is under a point of the app's document. Blitz
//! forwards a pointer move into a frame's document itself, but tells the app nothing about it;
//! ds-blitz hit-tests the same way (the frame's offset in its parent, its own scroll) so it can
//! say when the pointer comes onto and leaves a link, for the app's link pill, and which image a
//! click ended on.

use crate::frame_anchor::{LinkFacts, anchor_at};
use crate::frame_img::{ImageFacts, image_at};
use crate::origin::FrameId;
use blitz_dom::{BaseDocument, NodeId};
use ds::prelude::*;

/// A link under the pointer, inside a frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkUnder {
    /// The frame's document.
    pub(crate) frame: FrameId,
    /// The anchor element in it: the same link while this stays the same.
    pub(crate) anchor: NodeId,
    /// What the link says.
    pub(crate) facts: LinkFacts,
}

/// An image under the pointer, inside a frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageUnder {
    /// The frame's document.
    pub(crate) frame: FrameId,
    /// What the image shows.
    pub(crate) facts: ImageFacts,
}

/// What a hit node in a frame's document is, if it is the thing looked for.
type Find<'a, T> = &'a dyn Fn(&BaseDocument, NodeId) -> Option<T>;

/// The link under `at` (the app document's client coordinates) inside a frame, if any. A link
/// in the app's own document is the app's to track.
pub fn link_under(top: &BaseDocument, at: Point) -> Option<LinkUnder> {
    let (frame, (anchor, facts)) = descend(top, page(top, at), None, &anchor_at)?;
    Some(LinkUnder {
        frame,
        anchor,
        facts,
    })
}

/// The image under `at` (the app document's client coordinates) inside a frame, if any: an
/// `img` with a `src` and no link above it.
pub fn image_under(top: &BaseDocument, at: Point) -> Option<ImageUnder> {
    let (frame, facts) = descend(top, page(top, at), None, &image_at)?;
    Some(ImageUnder { frame, facts })
}

/// `at` in `top`'s page coordinates.
fn page(top: &BaseDocument, at: Point) -> (f32, f32) {
    let scroll = top.viewport_scroll();
    (at.x.0 + scroll.x as f32, at.y.0 + scroll.y as f32)
}

/// Hit-test `doc` at page point `(x, y)`, going into the frame hit if there is one, and `find`
/// what the node hit inside a frame is; `frame` is the frame `doc` is, or `None` for the app's
/// document.
fn descend<T>(
    doc: &BaseDocument,
    (x, y): (f32, f32),
    frame: Option<FrameId>,
    find: Find<'_, T>,
) -> Option<(FrameId, T)> {
    let hit = doc.hit(x, y)?;
    let node = doc.get_node(hit.node_id)?;
    match node.subdoc() {
        Some(sub) if crate::layout_chain::placeable(doc, node.id) => {
            let offset = node.absolute_position(0.0, 0.0);
            let inner = sub.inner();
            let scroll = inner.viewport_scroll();
            let inside = (
                x - offset.x + scroll.x as f32,
                y - offset.y + scroll.y as f32,
            );
            descend(&inner, inside, Some(FrameId::of(sub.id())), find)
        }
        // A frame whose layout chain lost a box cannot be placed (`layout_chain`): nothing.
        Some(_) => None,
        None => {
            let found = find(doc, hit.node_id)?;
            Some((frame?, found))
        }
    }
}
