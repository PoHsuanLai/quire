//! What a press would land on: the harness's hit test, the one Blitz runs for the pointer, with
//! CSS transforms and `pointer-events` applied (the layout rects [`Harness::rect`] reads leave
//! transforms out, FINDINGS "Native focus").

use crate::harness::{Harness, first};
use blitz_dom::{BaseDocument, NodeId};
use ds::Point;

impl Harness {
    /// Whether a press at `at` would target the first element matching `selector` or something
    /// inside it: `document.elementFromPoint` and its ancestors.
    pub fn hits(&self, at: Point, selector: &str) -> bool {
        self.with_doc(|doc| {
            let Some(wanted) = first(doc, selector) else {
                return false;
            };
            let Some(hit) = doc.element_from_point(at.x.0, at.y.0) else {
                return false;
            };
            chain(doc, hit).contains(&wanted)
        })
    }
}

/// `node` and every ancestor, innermost first.
fn chain(doc: &BaseDocument, node: NodeId) -> Vec<NodeId> {
    std::iter::successors(Some(node), |id| doc.get_node(*id).and_then(|n| n.parent)).collect()
}
