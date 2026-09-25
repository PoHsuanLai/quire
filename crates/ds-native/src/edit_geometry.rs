//! Caret and selection boxes from an inline root's parley layout, in the window's logical
//! pixels. parley measures in the layout's own units (logical pixels times the display scale,
//! `Layout::scale`), from the inline root's content box.

use crate::edit_locate::{Order, Side, Spot, order_of};
use crate::edit_tree::Segment;
use blitz_dom::{BaseDocument, Node, NodeId};
use ds::{Point, Px, Rect, Size};
use parley::{Affinity, BoundingBox, Cursor, Selection};

/// The caret's box at `spot`: zero width, the height of its line.
pub(crate) fn caret(doc: &BaseDocument, spot: Spot) -> Option<Rect> {
    match spot {
        Spot::Text { root, byte } => {
            let node = doc.get_node(root)?;
            let layout = &node.element_data()?.inline_layout_data.as_ref()?.layout;
            let cursor = Cursor::from_byte_index(layout, byte, Affinity::Downstream);
            Some(to_logical(
                doc,
                node,
                layout.scale(),
                cursor.geometry(layout, 0.0),
            ))
        }
        Spot::Atom { atom, side } => {
            let border = border_box(doc, doc.get_node(atom)?);
            let x = match side {
                Side::Before => border.origin.x,
                Side::After => border.origin.x + border.size.width,
            };
            Some(line_at(x, border.origin.y, border.size.height))
        }
        Spot::Empty { element } => {
            let node = doc.get_node(element)?;
            let origin = content_origin(doc, node);
            let height = node.unrounded_layout().content_box_height();
            Some(line_at(Px(origin.0), Px(origin.1), Px(height)))
        }
    }
}

/// The boxes between `from` and `to` (either first), in reading order over `segments`: each
/// line of text they cover and each whole inline box or block atom between them.
pub(crate) fn selection(
    doc: &BaseDocument,
    segments: &[Segment],
    from: Spot,
    to: Spot,
) -> Option<Vec<Rect>> {
    let (a, b) = (order_of(doc, segments, from)?, order_of(doc, segments, to)?);
    let (low, high) = if a <= b { (a, b) } else { (b, a) };
    Some(
        segments
            .iter()
            .enumerate()
            .take(high.0 + 1)
            .skip(low.0)
            .flat_map(|(index, &segment)| segment_rects(doc, index, segment, low, high))
            .collect(),
    )
}

fn segment_rects(
    doc: &BaseDocument,
    index: usize,
    segment: Segment,
    low: Order,
    high: Order,
) -> Vec<Rect> {
    let Some(node) = doc.get_node(segment.node()) else {
        return Vec::new();
    };
    let covers = |before: Order, after: Order| low <= before && after <= high;
    match segment {
        Segment::Atom(_) if covers((index, 0, 0), (index, 0, 2)) => vec![border_box(doc, node)],
        Segment::Atom(_) => Vec::new(),
        Segment::Text(_) => {
            let Some(text) = node
                .element_data()
                .and_then(|element| element.inline_layout_data.as_ref())
            else {
                return Vec::new();
            };
            let layout = &text.layout;
            let start = if index == low.0 { low.1 } else { 0 };
            let end = if index == high.0 {
                high.1
            } else {
                text.text.len()
            };
            let lines = Selection::new(
                Cursor::from_byte_index(layout, start, Affinity::Downstream),
                Cursor::from_byte_index(layout, end, Affinity::Upstream),
            )
            .geometry(layout)
            .into_iter()
            .map(|(bounds, _line)| to_logical(doc, node, layout.scale(), bounds));
            let boxes = layout
                .inline_boxes()
                .iter()
                .filter(|b| covers((index, b.index, 0), (index, b.index, 2)))
                .filter_map(|b| doc.get_node(NodeId::from_u64(b.id)))
                .map(|boxed| border_box(doc, boxed));
            lines.chain(boxes).collect()
        }
    }
}

/// A parley box of inline root `node`'s layout, at display scale `scale`, in logical pixels.
fn to_logical(doc: &BaseDocument, node: &Node, scale: f32, bounds: BoundingBox) -> Rect {
    let (x, y) = content_origin(doc, node);
    let scale = f64::from(scale.max(f32::EPSILON));
    Rect {
        origin: Point {
            x: Px(x + (bounds.x0 / scale) as f32),
            y: Px(y + (bounds.y0 / scale) as f32),
        },
        size: Size {
            width: Px(((bounds.x1 - bounds.x0) / scale) as f32),
            height: Px(((bounds.y1 - bounds.y0) / scale) as f32),
        },
    }
}

/// A zero-width line box.
fn line_at(x: Px, y: Px, height: Px) -> Rect {
    Rect {
        origin: Point { x, y },
        size: Size {
            width: Px(0.0),
            height,
        },
    }
}

/// Where `node`'s border box starts, in the window's logical pixels: as
/// `get_client_bounding_rect` (and so `ds::HostMeasure`) reads it, from the unrounded layout, so
/// a caret rect and a measured container agree to the 64th of a pixel.
fn border_origin(doc: &BaseDocument, node: &Node) -> (f32, f32) {
    let at = node.unrounded_absolute_position(0.0, 0.0);
    let scroll = doc.viewport_scroll();
    (
        snap(f64::from(at.x) - scroll.x),
        snap(f64::from(at.y) - scroll.y),
    )
}

/// A length to the layout unit (a 64th of a pixel), as Blitz's client rects are.
fn snap(value: f64) -> f32 {
    ((value * 64.0).round() / 64.0) as f32
}

/// Where `node`'s content box starts, in the window's logical pixels.
pub(crate) fn content_origin(doc: &BaseDocument, node: &Node) -> (f32, f32) {
    let (x, y) = border_origin(doc, node);
    let layout = node.unrounded_layout();
    (
        x + layout.border.left + layout.padding.left,
        y + layout.border.top + layout.padding.top,
    )
}

/// `node`'s border box, in the window's logical pixels.
pub(crate) fn border_box(doc: &BaseDocument, node: &Node) -> Rect {
    let (x, y) = border_origin(doc, node);
    let size = node.unrounded_layout().size;
    Rect {
        origin: Point { x: Px(x), y: Px(y) },
        size: Size {
            width: Px(snap(f64::from(size.width))),
            height: Px(snap(f64::from(size.height))),
        },
    }
}

/// `node`'s content box, in the window's logical pixels.
pub(crate) fn content_box(doc: &BaseDocument, node: &Node) -> Rect {
    let (x, y) = content_origin(doc, node);
    let layout = node.unrounded_layout();
    Rect {
        origin: Point { x: Px(x), y: Px(y) },
        size: Size {
            width: Px(layout.content_box_width()),
            height: Px(layout.content_box_height()),
        },
    }
}
