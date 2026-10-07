//! What a laid-out document shows, as the probe file's lines.

use super::name::name_of;
use super::role::{attr, role_of};
use blitz_dom::{BaseDocument, ElementData, Node};

/// One line per control with a role and a box, in document order.
pub fn snapshot(doc: &BaseDocument) -> String {
    let mut out = String::new();
    walk(doc, doc.root_node(), &mut out);
    out
}

fn walk(doc: &BaseDocument, node: &Node, out: &mut String) {
    if let Some(element) = node.element_data() {
        if attr(element, "aria-hidden") == Some("true") {
            return;
        }
        line(doc, node, element, out);
    }
    for child in &node.children {
        if let Some(child) = doc.get_node(*child) {
            walk(doc, child, out);
        }
    }
}

fn line(doc: &BaseDocument, node: &Node, element: &ElementData, out: &mut String) {
    let Some(role) = role_of(element) else { return };
    let Some(rect) = doc.get_client_bounding_rect(node.id) else {
        return;
    };
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return;
    }
    let state = state_of(element);
    out.push_str(&format!(
        "{role}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{state}\t{}\n",
        rect.x,
        rect.y,
        rect.width,
        rect.height,
        name_of(doc, node)
    ));
}

/// The words for what `element` says about its state, `-` for none.
fn state_of(element: &ElementData) -> String {
    let words: Vec<&str> = [
        ("aria-checked", "checked"),
        ("aria-pressed", "pressed"),
        ("aria-selected", "selected"),
        ("aria-expanded", "expanded"),
        ("aria-disabled", "disabled"),
    ]
    .into_iter()
    .filter(|(name, _)| attr(element, name) == Some("true"))
    .map(|(_, word)| word)
    .collect();
    match words.is_empty() {
        true => "-".to_owned(),
        false => words.join(" "),
    }
}
