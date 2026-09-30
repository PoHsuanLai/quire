//! Finding the app's addressable elements (`data-edit-node`) and their text in a Blitz
//! document, and the order an edit surface's laid-out text runs in. Reads only; every function
//! takes the document the caller already holds.

use blitz_dom::{BaseDocument, LocalName, Node, NodeId};
use ds::{EDIT_KIND_ATTR, EDIT_NODE_ATTR, EditKind, EditNode};

/// A stretch of a surface in reading order: an inline root's laid-out text, or a block atom.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Segment {
    /// The text of an inline formatting context (a paragraph, usually).
    Text(NodeId),
    /// An atom laid out as a block of its own (mailo's quoted reply).
    Atom(NodeId),
}

impl Segment {
    pub(crate) fn node(self) -> NodeId {
        match self {
            Segment::Text(id) | Segment::Atom(id) => id,
        }
    }
}

/// The addressable name and kind of `node`, if the app marked it.
pub(crate) fn mark_of(node: &Node) -> Option<(EditNode, EditKind)> {
    let element = node.element_data()?;
    let key = element.attr(LocalName::from(EDIT_NODE_ATTR))?;
    let kind = EditKind::from_slug(element.attr(LocalName::from(EDIT_KIND_ATTR)));
    Some((EditNode(key.to_owned()), kind))
}

/// The element marked `key` inside `surface`, first in document order.
pub(crate) fn find_marked(
    doc: &BaseDocument,
    surface: NodeId,
    key: &EditNode,
) -> Option<(NodeId, EditKind)> {
    let node = doc.get_node(surface)?;
    node.children.iter().find_map(|&child| {
        let child_node = doc.get_node(child)?;
        match mark_of(child_node) {
            Some((found, kind)) if found == *key => Some((child, kind)),
            _ => find_marked(doc, child, key),
        }
    })
}

/// The nearest marked element at or above `from`, stopping at `surface` (never outside it).
pub(crate) fn nearest_marked(doc: &BaseDocument, from: NodeId, surface: NodeId) -> Option<NodeId> {
    let mut at = Some(from);
    while let Some(id) = at {
        if id == surface {
            return None;
        }
        let node = doc.get_node(id)?;
        if mark_of(node).is_some() {
            return Some(id);
        }
        at = node.parent;
    }
    None
}

/// Whether `node` is `ancestor` or inside it.
pub(crate) fn is_within(doc: &BaseDocument, node: NodeId, ancestor: NodeId) -> bool {
    let mut at = Some(node);
    while let Some(id) = at {
        if id == ancestor {
            return true;
        }
        at = doc.get_node(id).and_then(|found| found.parent);
    }
    false
}

/// The text nodes that are `marked`'s own: those whose nearest marked ancestor it is, in
/// document order, skipping anything not displayed.
pub(crate) fn own_texts(doc: &BaseDocument, marked: NodeId) -> Vec<NodeId> {
    let mut found = Vec::new();
    collect_own(doc, marked, &mut found);
    found
}

fn collect_own(doc: &BaseDocument, parent: NodeId, found: &mut Vec<NodeId>) {
    let Some(node) = doc.get_node(parent) else {
        return;
    };
    for &child in node.children.iter() {
        let Some(child_node) = doc.get_node(child) else {
            continue;
        };
        if child_node.is_text_node() {
            found.push(child);
        } else if mark_of(child_node).is_none() && !is_hidden(child_node) {
            collect_own(doc, child, found);
        }
    }
}

/// The text nodes laid out in inline root `root`, in layout order: its descendants, less
/// inline boxes (laid out on their own) and anything not displayed. Generated content is left
/// to the alignment to step over.
pub(crate) fn root_texts(doc: &BaseDocument, root: NodeId) -> Vec<NodeId> {
    let boxes: Vec<u64> = doc
        .get_node(root)
        .and_then(|node| node.element_data())
        .and_then(|element| element.inline_layout_data.as_ref())
        .map(|text| text.layout.inline_boxes().iter().map(|b| b.id).collect())
        .unwrap_or_default();
    let mut found = Vec::new();
    collect_laid(doc, root, &boxes, &mut found);
    found
}

fn collect_laid(doc: &BaseDocument, parent: NodeId, boxes: &[u64], found: &mut Vec<NodeId>) {
    let Some(node) = doc.get_node(parent) else {
        return;
    };
    for &child in node.children.iter() {
        let Some(child_node) = doc.get_node(child) else {
            continue;
        };
        if child_node.is_text_node() {
            found.push(child);
        } else if !boxes.contains(&child.as_u64()) && !is_hidden(child_node) {
            collect_laid(doc, child, boxes, found);
        }
    }
}

/// A text node's content.
pub(crate) fn text_of(doc: &BaseDocument, id: NodeId) -> &str {
    doc.get_node(id)
        .and_then(|node| node.text_data())
        .map_or("", |text| text.content.as_str())
}

/// The inline root laying `node` out as an inline box, and the layout byte the box sits at.
pub(crate) fn inline_box_of(doc: &BaseDocument, node: &Node) -> Option<(NodeId, usize)> {
    let parent = doc.get_node(node.layout_parent.get()?)?;
    let root = parent.inline_root_ancestor()?;
    let layout = &root.element_data()?.inline_layout_data.as_ref()?.layout;
    layout
        .inline_boxes()
        .iter()
        .find(|b| b.id == node.id.as_u64())
        .map(|b| (root.id, b.index))
}

/// Whether `node` has an inline layout of its own.
pub(crate) fn has_text_layout(node: &Node) -> bool {
    node.flags.is_inline_root()
        && node
            .element_data()
            .is_some_and(|element| element.inline_layout_data.is_some())
}

/// `surface`'s stretches in reading order.
pub(crate) fn segments(doc: &BaseDocument, surface: NodeId) -> Vec<Segment> {
    let mut found = Vec::new();
    if let Some(node) = doc.get_node(surface)
        && has_text_layout(node)
    {
        found.push(Segment::Text(surface));
    }
    collect_segments(doc, surface, &mut found);
    found
}

fn collect_segments(doc: &BaseDocument, parent: NodeId, found: &mut Vec<Segment>) {
    let Some(node) = doc.get_node(parent) else {
        return;
    };
    for &child in node.children.iter() {
        let Some(child_node) = doc.get_node(child) else {
            continue;
        };
        let segment = segment_at(doc, child_node);
        if let Some(segment) = segment
            && !found.contains(&segment)
        {
            found.push(segment);
        }
        let atom = matches!(mark_of(child_node), Some((_, EditKind::Atom)));
        if !child_node.is_text_node() && !atom && !is_hidden(child_node) {
            collect_segments(doc, child, found);
        }
    }
}

/// The stretch `node` starts, if it starts one.
fn segment_at(doc: &BaseDocument, node: &Node) -> Option<Segment> {
    if node.is_text_node() {
        return node
            .inline_root_ancestor()
            .map(|root| Segment::Text(root.id));
    }
    if is_hidden(node) {
        return None;
    }
    match mark_of(node) {
        Some((_, EditKind::Atom)) if inline_box_of(doc, node).is_none() => {
            Some(Segment::Atom(node.id))
        }
        Some((_, EditKind::Atom)) => None,
        _ if has_text_layout(node) => Some(Segment::Text(node.id)),
        _ => None,
    }
}

/// Whether `node` is not displayed (`display: none`), so none of its text is laid out.
pub(crate) fn is_hidden(node: &Node) -> bool {
    node.primary_styles()
        .is_some_and(|style| style.clone_display().is_none())
}
