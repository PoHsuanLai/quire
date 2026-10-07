//! The accessible name of an element, as far as a scenario needs it.

use super::role::attr;
use blitz_dom::{BaseDocument, Node, NodeData};

/// `node`'s name: `aria-label`, else an image's `alt`, else its visible text (what `aria-hidden`
/// does not hide) with runs of white space made one space.
pub(super) fn name_of(doc: &BaseDocument, node: &Node) -> String {
    let labelled = node.element_data().and_then(|element| {
        [attr(element, "aria-label"), attr(element, "alt")]
            .into_iter()
            .flatten()
            .find(|text| !text.trim().is_empty())
    });
    match labelled {
        Some(text) => squash(text),
        None => {
            let mut text = String::new();
            visible_text(doc, node, &mut text);
            squash(&text)
        }
    }
}

fn visible_text(doc: &BaseDocument, node: &Node, out: &mut String) {
    match &node.data {
        NodeData::Text(text) => out.push_str(&text.content),
        NodeData::Element(element) if attr(element, "aria-hidden") != Some("true") => {
            for child in &node.children {
                if let Some(child) = doc.get_node(*child) {
                    visible_text(doc, child, out);
                }
            }
        }
        NodeData::AnonymousBlock(_) => {
            for child in &node.children {
                if let Some(child) = doc.get_node(*child) {
                    visible_text(doc, child, out);
                }
            }
        }
        _ => {}
    }
}

/// `text` with white space runs made one space, and the ends trimmed; a tab or newline never
/// reaches the file's columns.
fn squash(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
