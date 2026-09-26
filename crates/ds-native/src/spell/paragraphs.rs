//! An edit surface's paragraphs as the checker reads them: every addressable text element's own
//! text (the same text a `TextPosition`'s offsets count, `edit_tree::own_texts`), with the
//! stretches inside code elements marked as never checked.

use crate::edit_tree::{is_hidden, mark_of};
use blitz_dom::{BaseDocument, NodeId};
use ds::{EditKind, Paragraph, Span};

/// Elements whose text is code, not prose.
const CODE: &[&str] = &["code", "pre", "kbd", "samp", "tt", "var"];

/// Whether the text inside an element is prose or code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Prose {
    Yes,
    Code,
}

/// Every addressable text element inside `surface`, in document order.
pub(crate) fn paragraphs(doc: &BaseDocument, surface: NodeId) -> Vec<Paragraph> {
    let mut found = Vec::new();
    collect(doc, surface, &mut found);
    found
}

fn collect(doc: &BaseDocument, parent: NodeId, found: &mut Vec<Paragraph>) {
    let Some(node) = doc.get_node(parent) else {
        return;
    };
    for &child in node.children.iter() {
        let Some(child_node) = doc.get_node(child) else {
            continue;
        };
        if is_hidden(child_node) {
            continue;
        }
        if let Some((key, EditKind::Text)) = mark_of(child_node) {
            let mut paragraph = Paragraph {
                node: key,
                text: String::new(),
                skips: Vec::new(),
            };
            own(doc, child, prose_of(doc, child), &mut paragraph);
            found.push(paragraph);
        }
        collect(doc, child, found);
    }
}

/// Append the text nodes whose nearest addressable ancestor is `parent`'s paragraph.
fn own(doc: &BaseDocument, parent: NodeId, prose: Prose, paragraph: &mut Paragraph) {
    let Some(node) = doc.get_node(parent) else {
        return;
    };
    for &child in node.children.iter() {
        let Some(child_node) = doc.get_node(child) else {
            continue;
        };
        if let Some(text) = child_node.text_data() {
            let start = paragraph.text.len();
            paragraph.text.push_str(&text.content);
            if prose == Prose::Code {
                paragraph.skips.push(Span::new(start, paragraph.text.len()));
            }
        } else if mark_of(child_node).is_none() && !is_hidden(child_node) {
            let inner = match prose {
                Prose::Code => Prose::Code,
                Prose::Yes => prose_of(doc, child),
            };
            own(doc, child, inner, paragraph);
        }
    }
}

/// Code when the element is a code element (a paragraph may itself be a `pre`).
fn prose_of(doc: &BaseDocument, id: NodeId) -> Prose {
    let code = doc
        .get_node(id)
        .and_then(|node| node.element_data())
        .is_some_and(|element| CODE.contains(&&*element.name.local));
    match code {
        true => Prose::Code,
        false => Prose::Yes,
    }
}
