//! The app's text position under a point: over an atom, the side of it nearer the point; over
//! text or beside it (padding, a gap between paragraphs, past a line's end), the caret position
//! nearest, in the stretch of text nearest the point vertically.

use crate::edit_geometry::{border_box, content_box, content_origin};
use crate::edit_locate::{Side, named, position_at};
use crate::edit_tree::{Segment, is_within, mark_of, nearest_marked, segments};
use blitz_dom::{BaseDocument, NodeId};
use ds::{EditKind, Point, Rect, TextPosition};
use parley::Cursor;

/// The position under `at` in `surface`, or `None` where nothing addressable is near.
pub(crate) fn hit(doc: &BaseDocument, surface: NodeId, at: Point) -> Option<TextPosition> {
    let marked = doc
        .hit(at.x.0, at.y.0)
        .and_then(|found| nearest_marked(doc, found.node_id, surface));
    if let Some(marked) = marked
        && let Some((_, EditKind::Atom)) = mark_of(doc.get_node(marked)?)
    {
        return atom_side(doc, marked, at);
    }
    let all = segments(doc, surface);
    let candidates: Vec<Segment> = match marked {
        Some(marked) => all
            .iter()
            .copied()
            .filter(|segment| {
                is_within(doc, segment.node(), marked) || is_within(doc, marked, segment.node())
            })
            .collect(),
        None => all,
    };
    match nearest(doc, &candidates, at) {
        Some(Segment::Text(root)) => in_text(doc, surface, root, at),
        Some(Segment::Atom(atom)) => atom_side(doc, atom, at),
        None => named(doc, marked?, 0),
    }
}

/// The side of `atom` nearer `at`.
fn atom_side(doc: &BaseDocument, atom: NodeId, at: Point) -> Option<TextPosition> {
    let border = border_box(doc, doc.get_node(atom)?);
    let middle = border.origin.x.0 + border.size.width.0 / 2.0;
    let side = if at.x.0 < middle {
        Side::Before
    } else {
        Side::After
    };
    named(doc, atom, side.offset())
}

/// The caret position nearest `at` in inline root `root`'s text.
fn in_text(doc: &BaseDocument, surface: NodeId, root: NodeId, at: Point) -> Option<TextPosition> {
    let node = doc.get_node(root)?;
    let layout = &node.element_data()?.inline_layout_data.as_ref()?.layout;
    let (x, y) = content_origin(doc, node);
    let scale = layout.scale();
    let cursor = Cursor::from_point(layout, (at.x.0 - x) * scale, (at.y.0 - y) * scale);
    position_at(doc, surface, root, cursor.index())
}

/// The stretch whose box is vertically nearest `at` (containing it is nearest), the first of
/// equals.
fn nearest(doc: &BaseDocument, candidates: &[Segment], at: Point) -> Option<Segment> {
    candidates
        .iter()
        .copied()
        .filter_map(|segment| {
            let node = doc.get_node(segment.node())?;
            let rect = match segment {
                Segment::Text(_) => content_box(doc, node),
                Segment::Atom(_) => border_box(doc, node),
            };
            Some((distance(rect, at), segment))
        })
        .min_by(|(a, _), (b, _)| a.total_cmp(b))
        .map(|(_, segment)| segment)
}

/// How far `at` is above or below `rect`; 0 inside its vertical span.
fn distance(rect: Rect, at: Point) -> f32 {
    let top = rect.origin.y.0;
    let bottom = top + rect.size.height.0;
    (top - at.y.0).max(at.y.0 - bottom).max(0.0)
}
