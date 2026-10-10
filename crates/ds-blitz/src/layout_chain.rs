//! Whether a node can be placed: every box on its layout chain is still in the document.
//!
//! Blitz places a node by walking `layout_parent` and unwraps each step. A relayout can free a
//! box (an anonymous block) that a still-mounted node names as its layout parent, and reading
//! that node's position then panics inside Blitz (a hover anchor followed after a click,
//! 2026-10-10). Every position read in this crate asks here first and reads nothing instead.
//! The fork's own fix (stop the walk at a missing parent) waits for a toolchain batch.

use blitz_dom::{BaseDocument, NodeId};

/// A chain longer than this is a cycle, which a laid-out tree never has: stop instead of spinning.
const DEEPEST: usize = 4096;

/// Whether `node` and every box above it on its layout chain are in `doc`.
pub(crate) fn placeable(doc: &BaseDocument, node: NodeId) -> bool {
    let mut at = node;
    for _ in 0..DEEPEST {
        let Some(found) = doc.get_node(at) else {
            return false;
        };
        match found.layout_parent.get() {
            Some(parent) => at = parent,
            None => return true,
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::placeable;
    use blitz_dom::{DocumentConfig, NodeId};
    use blitz_html::HtmlDocument;

    #[test]
    fn a_node_whose_layout_parent_was_freed_is_not_placed() {
        let mut doc = HtmlDocument::from_html(
            "<div><p id=\"text\">words</p></div>",
            DocumentConfig::default(),
        );
        doc.resolve(0.0);
        let text = doc
            .query_selector("#text")
            .ok()
            .flatten()
            .expect("the paragraph");
        assert!(placeable(&doc, text));
        let freed = NodeId::from_u64(u64::MAX - 1);
        doc.get_node(text)
            .expect("the paragraph")
            .layout_parent
            .set(Some(freed));
        assert!(!placeable(&doc, text), "a freed box on the chain");
    }
}
