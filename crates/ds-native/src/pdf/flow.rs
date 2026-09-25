//! The flow pagination reads, collected from a laid-out document: every line box, replaced
//! element, table row and marked box as a keep-together span, each text block's first and last
//! two lines as orphan and widow spans, and every forced break.
//!
//! The markers are attributes, not CSS, because Blitz's style engine drops the fragmentation
//! properties: `data-break-before="page"` starts a page at the element, and
//! `data-break-inside="avoid"` keeps the element on one page if it fits on one.

use crate::pdf::paginate::{Flow, Span};
use blitz_dom::{BaseDocument, LocalName, Node};

/// Elements that are one picture: never cut through while they fit on a page.
const REPLACED: &[&str] = &["img", "svg", "canvas", "video", "iframe", "tr"];

/// The flow of `doc` as laid out now.
pub(crate) fn collect(doc: &BaseDocument) -> Flow {
    let mut flow = Flow {
        height: doc.root_element().final_layout().size.height,
        ..Flow::default()
    };
    for (_, node) in doc.tree().iter() {
        let Some(element) = node.element_data() else {
            continue;
        };
        let size = node.final_layout().size;
        let top = node.absolute_position(0.0, 0.0).y;
        let border_box = Span {
            top,
            bottom: top + size.height,
        };
        flow.height = flow.height.max(border_box.bottom);
        if element.attr(LocalName::from("data-break-before")) == Some("page") {
            flow.breaks.push(border_box.top);
        }
        let marked = element.attr(LocalName::from("data-break-inside")) == Some("avoid");
        let replaced = REPLACED.iter().any(|&tag| &*element.name.local == tag);
        if (marked || replaced) && size.height > 0.0 {
            flow.keeps.push(border_box);
        }
        if element.inline_layout_data.is_some() {
            flow.keeps.extend(text_block(node));
        }
    }
    flow
}

/// The line boxes of the inline content in `node`, and its orphan and widow spans.
fn text_block(node: &Node) -> Vec<Span> {
    let Some(text) = node
        .element_data()
        .and_then(|e| e.inline_layout_data.as_ref())
    else {
        return Vec::new();
    };
    let layout = node.final_layout();
    let top = node.absolute_position(0.0, 0.0).y + layout.padding.top + layout.border.top;
    let lines: Vec<Span> = text
        .layout
        .lines()
        .map(|line| {
            let metrics = line.metrics();
            Span {
                top: top + metrics.block_min_coord,
                bottom: top + metrics.block_max_coord,
            }
        })
        .collect();
    let pairs = match lines.as_slice() {
        [first, second, .., before_last, last] => vec![
            Span {
                top: first.top,
                bottom: second.bottom,
            },
            Span {
                top: before_last.top,
                bottom: last.bottom,
            },
        ],
        [first, last] => vec![Span {
            top: first.top,
            bottom: last.bottom,
        }],
        _ => Vec::new(),
    };
    [lines, pairs].concat()
}
