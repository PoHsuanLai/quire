//! Between the app's positions (`data-edit-node` + offset) and the layout's (an inline root's
//! text byte, or a side of an atom), and the reading order selection rects are taken in.

use crate::edit_align::{Stop, align, to_dom, to_layout};
use crate::edit_tree::{
    Segment, find_marked, has_text_layout, inline_box_of, mark_of, nearest_marked, own_texts,
    root_texts, text_of,
};
use blitz_dom::{BaseDocument, NodeId};
use ds::{EditKind, TextPosition};

/// A position as the layout knows it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Spot {
    /// `byte` into inline root `root`'s laid-out text.
    Text { root: NodeId, byte: usize },
    /// A side of an atom.
    Atom { atom: NodeId, side: Side },
    /// A marked element with no laid-out text (an empty block): its content box's start.
    Empty { element: NodeId },
}

/// Which side of an atom.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Side {
    Before,
    After,
}

impl Side {
    /// The side an offset into an atom names: 0 is before, anything else after.
    pub(crate) fn of_offset(offset: usize) -> Self {
        if offset == 0 {
            Side::Before
        } else {
            Side::After
        }
    }

    pub(crate) fn offset(self) -> usize {
        match self {
            Side::Before => 0,
            Side::After => 1,
        }
    }
}

/// Where `position` is in `surface`'s layout.
pub(crate) fn resolve(
    doc: &BaseDocument,
    surface: NodeId,
    position: &TextPosition,
) -> Option<Spot> {
    let (marked, kind) = find_marked(doc, surface, &position.node)?;
    if kind == EditKind::Atom {
        return Some(Spot::Atom {
            atom: marked,
            side: Side::of_offset(position.offset.0),
        });
    }
    let texts = own_texts(doc, marked);
    let Some((text, local)) = text_at(doc, &texts, position.offset.0) else {
        let node = doc.get_node(marked)?;
        return Some(match has_text_layout(node) {
            true => Spot::Text {
                root: marked,
                byte: 0,
            },
            false => Spot::Empty { element: marked },
        });
    };
    let root = doc.get_node(text)?.inline_root_ancestor()?.id;
    let (laid, stops) = aligned(doc, root);
    let index = laid.iter().position(|&id| id == text)?;
    Some(Spot::Text {
        root,
        byte: to_layout(&stops[index], local),
    })
}

/// The text node holding `offset` bytes into `texts` taken together, and the offset into it: a
/// boundary is the start of the later text, the very end the last text's end.
fn text_at(doc: &BaseDocument, texts: &[NodeId], offset: usize) -> Option<(NodeId, usize)> {
    let mut left = offset;
    for (index, &text) in texts.iter().enumerate() {
        let length = text_of(doc, text).len();
        if left < length || index + 1 == texts.len() {
            return Some((text, left.min(length)));
        }
        left -= length;
    }
    None
}

/// The app's position at layout byte `byte` of inline root `root`, or `None` where the text
/// there is not inside anything marked.
pub(crate) fn position_at(
    doc: &BaseDocument,
    surface: NodeId,
    root: NodeId,
    byte: usize,
) -> Option<TextPosition> {
    let (laid, stops) = aligned(doc, root);
    let flat: Vec<(usize, Stop)> = stops
        .iter()
        .enumerate()
        .flat_map(|(index, text)| text.iter().map(move |&stop| (index, stop)))
        .collect();
    let Some((index, dom)) = to_dom(&flat, byte) else {
        let marked = nearest_marked(doc, root, surface)?;
        return named(doc, marked, 0);
    };
    let text = laid[index];
    let marked = nearest_marked(doc, text, surface)?;
    if matches!(mark_of(doc.get_node(marked)?), Some((_, EditKind::Atom))) {
        return named(doc, marked, 0);
    }
    let before: usize = own_texts(doc, marked)
        .iter()
        .take_while(|&&id| id != text)
        .map(|&id| text_of(doc, id).len())
        .sum();
    named(doc, marked, before + dom)
}

/// The position `offset` into marked element `marked`.
pub(crate) fn named(doc: &BaseDocument, marked: NodeId, offset: usize) -> Option<TextPosition> {
    let (key, _) = mark_of(doc.get_node(marked)?)?;
    Some(TextPosition::new(key.0, offset))
}

/// The text nodes of inline root `root` and each one's alignment to its layout text.
fn aligned(doc: &BaseDocument, root: NodeId) -> (Vec<NodeId>, Vec<Vec<Stop>>) {
    let laid = root_texts(doc, root);
    let layout = doc
        .get_node(root)
        .and_then(|node| node.element_data())
        .and_then(|element| element.inline_layout_data.as_ref())
        .map_or("", |text| text.text.as_str());
    let contents: Vec<&str> = laid.iter().map(|&id| text_of(doc, id)).collect();
    let stops = align(layout, &contents);
    (laid, stops)
}

/// Where a spot falls in reading order: its stretch, the byte in it, and a rank that puts an
/// atom's before-side ahead of text at the same byte and its after-side behind.
pub(crate) type Order = (usize, usize, u8);

/// `spot`'s place in reading order among `segments`, if it is in one.
pub(crate) fn order_of(doc: &BaseDocument, segments: &[Segment], spot: Spot) -> Option<Order> {
    let index_of = |wanted: Segment| segments.iter().position(|&segment| segment == wanted);
    match spot {
        Spot::Text { root, byte } => Some((index_of(Segment::Text(root))?, byte, 1)),
        Spot::Atom { atom, side } => {
            let rank = match side {
                Side::Before => 0,
                Side::After => 2,
            };
            match inline_box_of(doc, doc.get_node(atom)?) {
                Some((root, index)) => Some((index_of(Segment::Text(root))?, index, rank)),
                None => Some((index_of(Segment::Atom(atom))?, 0, rank)),
            }
        }
        Spot::Empty { .. } => None,
    }
}
