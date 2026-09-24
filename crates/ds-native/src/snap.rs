//! Snapping a laid-out document to the device pixel grid (FINDINGS "Pixel snapping").
//!
//! Blitz lays out in logical pixels and then rounds every box to whole *logical* pixels
//! (`taffy::round_layout`, called at the end of `BaseDocument::resolve_layout`). At a whole
//! scale that is the device grid too. At 1.25, 1.5 or 1.75 it is not: a box at logical y 11 starts
//! at device y 16.5, and a border that stylo had already snapped to one device pixel (0.667
//! logical px at 1.5) is rounded back up to a whole logical pixel, 1.5 device pixels. Either
//! way the line paints one full row and one half row.
//!
//! [`snap_to_device`] redoes that rounding on the device grid: the same cumulative rounding
//! taffy does (so neighbours still abut exactly), in units of one device pixel, with border
//! widths rounded on their own to whole device pixels and never below one. It then re-derives
//! what Blitz computed from the logical rounding (each node's transform, whose percentages read
//! the box size, and its scrollable overflow), and rounds a pure translation to whole device
//! pixels, so a `translateX(-50%)` of an odd width does not undo it. Hit testing and rect reads
//! use the same final layout, so they agree with the picture.
//!
//! It must run after every `resolve` and before painting, because every resolve re-runs taffy's
//! rounding. `ds-native`'s headless documents (`Harness`, `snapshot`) run it; a host that drives
//! its own documents (shell-host) calls it after each `resolve`. At a whole scale it does
//! nothing: Blitz's rounding is already right there, and every picture stays as it was.

use blitz_dom::{BaseDocument, NodeId};
use peniko::kurbo::{Affine, Rect};

/// Snap every box of `doc` to its viewport's device pixel grid; nothing at a whole scale.
pub fn snap_to_device(doc: &mut BaseDocument) {
    let scale = doc.viewport().scale_f64();
    if scale <= 0.0 || scale.fract() == 0.0 {
        return;
    }
    let grid = DeviceGrid(scale);
    let root = doc.root_element().id;
    place(doc, root, Origin::default(), grid);
    settle(doc, root, grid);
}

/// Device pixels per logical pixel, at a fractional scale.
#[derive(Debug, Clone, Copy, PartialEq)]
struct DeviceGrid(f64);

impl DeviceGrid {
    /// A logical coordinate moved to the nearest device pixel boundary, rounding halves up as
    /// taffy's own rounding does.
    fn snap(self, logical: f64) -> f64 {
        (logical * self.0 + 0.5).floor() / self.0
    }

    /// The snapped extent of `length` starting at `start`: the difference of its snapped ends,
    /// so adjacent boxes share an edge exactly.
    fn span(self, start: f64, length: f32) -> f32 {
        (self.snap(start + f64::from(length)) - self.snap(start)) as f32
    }

    /// A border width in whole device pixels, never below one when there is a border at all:
    /// the CSS rule for snapping a border width, kept whatever phase the box's edge lands on.
    fn line(self, width: f32) -> f32 {
        if width <= 0.0 {
            return 0.0;
        }
        let device = (f64::from(width) * self.0).round().max(1.0);
        (device / self.0) as f32
    }
}

/// A box's top-left corner in document coordinates, before snapping.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Origin {
    x: f64,
    y: f64,
}

/// Snap `id`'s final layout from its unrounded one, then its layout children's.
fn place(doc: &mut BaseDocument, id: NodeId, parent: Origin, grid: DeviceGrid) {
    let Some(node) = doc.get_node(id) else {
        return;
    };
    let exact = *node.unrounded_layout();
    let children = layout_children(doc, id);
    let at = Origin {
        x: parent.x + f64::from(exact.location.x),
        y: parent.y + f64::from(exact.location.y),
    };
    let (right, bottom) = (
        at.x + f64::from(exact.size.width),
        at.y + f64::from(exact.size.height),
    );
    let mut snapped = exact;
    snapped.location.x = (grid.snap(at.x) - grid.snap(parent.x)) as f32;
    snapped.location.y = (grid.snap(at.y) - grid.snap(parent.y)) as f32;
    snapped.size.width = grid.span(at.x, exact.size.width);
    snapped.size.height = grid.span(at.y, exact.size.height);
    snapped.border.left = grid.line(exact.border.left);
    snapped.border.right = grid.line(exact.border.right);
    snapped.border.top = grid.line(exact.border.top);
    snapped.border.bottom = grid.line(exact.border.bottom);
    snapped.padding.left = grid.span(at.x, exact.padding.left);
    snapped.padding.right = grid.span(right - f64::from(exact.padding.right), exact.padding.right);
    snapped.padding.top = grid.span(at.y, exact.padding.top);
    snapped.padding.bottom = grid.span(
        bottom - f64::from(exact.padding.bottom),
        exact.padding.bottom,
    );
    snapped.scrollbar_size.width = grid.line(exact.scrollbar_size.width);
    snapped.scrollbar_size.height = grid.line(exact.scrollbar_size.height);
    let overflow = exact.scrollable_overflow_rect;
    snapped.scrollable_overflow_rect.left = grid.span(at.x, overflow.left);
    snapped.scrollable_overflow_rect.right = grid.span(at.x, overflow.right);
    snapped.scrollable_overflow_rect.top = grid.span(at.y, overflow.top);
    snapped.scrollable_overflow_rect.bottom = grid.span(at.y, overflow.bottom);
    if let Some(node) = doc.get_node_mut(id) {
        *node.final_layout_mut() = snapped;
    }
    for child in children {
        place(doc, child, at, grid);
    }
}

/// `id`'s layout children: the boxes taffy laid out under it, in-flow and out-of-flow (Blitz
/// makes every box the containing block of its own out-of-flow children).
fn layout_children(doc: &BaseDocument, id: NodeId) -> Vec<NodeId> {
    doc.get_node(id)
        .and_then(|node| node.layout_children.borrow().as_ref().map(|c| c.to_vec()))
        .unwrap_or_default()
}

/// Re-derive `id`'s transform and scrollable overflow from its snapped layout, as Blitz's
/// `resolve_transforms` did from the logical one; the box's painted bounds in its parent.
fn settle(doc: &mut BaseDocument, id: NodeId, grid: DeviceGrid) -> Rect {
    let mut children = layout_children(doc, id);
    let Some(node) = doc.get_node_mut(id) else {
        return Rect::ZERO;
    };
    children.extend(node.before());
    children.extend(node.after());
    let transform = node
        .set_transform(grid.0 as f32)
        .map(|t| whole_translation(t, node));
    let layout = *node.final_layout();
    let scale = grid.0;
    let mut overflow = Rect::new(
        0.0,
        0.0,
        f64::from(layout.size.width) * scale,
        f64::from(layout.size.height) * scale,
    );
    for child in children {
        overflow = overflow.union(settle(doc, child, grid));
    }
    let Some(node) = doc.get_node_mut(id) else {
        return Rect::ZERO;
    };
    *node.scrollable_overflow_mut() = overflow;
    let at = Affine::translate((
        f64::from(layout.location.x) * scale,
        f64::from(layout.location.y) * scale,
    ));
    (at * transform.unwrap_or(Affine::IDENTITY)).transform_rect_bbox(overflow)
}

/// A pure translation moved to whole device pixels (and written back to the node); any other
/// transform (a scale, a rotation, a motion part-way through) is left as it is.
fn whole_translation(transform: Affine, node: &mut blitz_dom::Node) -> Affine {
    let [a, b, c, d, e, f] = transform.as_coeffs();
    if (a, b, c, d) != (1.0, 0.0, 0.0, 1.0) {
        return transform;
    }
    let whole = Affine::new([a, b, c, d, e.round(), f.round()]);
    if let Some(slot) = node.transform_mut().as_deref_mut() {
        *slot = whole;
    }
    whole
}

#[cfg(test)]
mod tests {
    use super::DeviceGrid;

    /// At 1.5 a device pixel is 2/3 of a logical one.
    const GRID: DeviceGrid = DeviceGrid(1.5);

    #[test]
    fn a_coordinate_lands_on_a_device_pixel() {
        for (logical, device) in [(11.0, 17.0), (10.0, 15.0), (0.4, 1.0), (0.3, 0.0)] {
            assert_eq!(GRID.snap(logical) * 1.5, device, "{logical}");
        }
    }

    #[test]
    fn a_border_is_whole_device_pixels_and_never_vanishes() {
        // Stylo's snapped 1 px border at 1.5 (40 of 60 app units), and one just under it.
        for (width, device) in [(2.0 / 3.0, 1.0), (0.66, 1.0), (1.0, 2.0), (0.1, 1.0)] {
            let got = f64::from(GRID.line(width)) * 1.5;
            assert!((got - device).abs() < 1e-5, "{width}: {got}");
        }
        assert_eq!(GRID.line(0.0), 0.0);
    }

    #[test]
    fn adjacent_spans_share_their_edge() {
        let (first, second) = (GRID.span(10.3, 7.4), GRID.span(17.7, 5.0));
        let end = GRID.snap(10.3) + f64::from(first) + f64::from(second);
        assert!((end - GRID.snap(22.7)).abs() < 1e-5);
    }
}
