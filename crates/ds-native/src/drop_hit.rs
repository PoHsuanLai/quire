//! The host's hit test for a file drag (`ds::HostFileDrop`): the element under the pointer, as
//! Blitz finds it for a press (`element_from_point`: transforms and `pointer-events` applied),
//! and the innermost registered drop target on its ancestor chain.

use crate::node_ref::NodeRef;
use blitz_dom::{BaseDocument, NodeId};
use dioxus::prelude::MountedData;
use ds::{DropHit, Point};
use std::rc::Rc;

/// The seam `launch` and the harness provide: `ds::HostFileDrop::new(drop_seam())`.
pub(crate) fn drop_seam() -> ds::HostFileDrop {
    ds::HostFileDrop::new(drop_hit)
}

/// The innermost of `targets` whose element is under `at` or an ancestor of what is.
fn drop_hit(targets: &[Rc<MountedData>], at: Point) -> DropHit {
    let nodes: Vec<Option<NodeRef>> = targets.iter().map(|target| NodeRef::of(target)).collect();
    let Some(any) = nodes.iter().flatten().next() else {
        return DropHit::Nothing;
    };
    any.read(|doc| innermost(doc, &nodes, at))
        .unwrap_or(DropHit::Busy)
}

fn innermost(doc: &BaseDocument, nodes: &[Option<NodeRef>], at: Point) -> DropHit {
    let Some(hit) = doc.element_from_point(at.x.0, at.y.0) else {
        return DropHit::Nothing;
    };
    chain(doc, hit)
        .find_map(|id| {
            nodes
                .iter()
                .position(|node| node.as_ref().is_some_and(|node| node.node == id))
        })
        .map_or(DropHit::Nothing, DropHit::Target)
}

/// `node` and every ancestor, innermost first.
fn chain(doc: &BaseDocument, node: NodeId) -> impl Iterator<Item = NodeId> + '_ {
    std::iter::successors(Some(node), |id| doc.get_node(*id).and_then(|n| n.parent))
}
