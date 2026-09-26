//! The host's reveal write (`ds::HostReveal`, sill Q340): scroll a list's own content the least
//! that shows one of its items. Blitz's `scroll_into_view` only ever scrolls the document's
//! viewport, so a palette row below a 360 px list stayed below it; this reads the item's place in
//! the list's content from the layout and sets the list's offset, nearest edge first
//! (`ds::nearest_scroll`), with no animation.

use crate::node_ref::{NodeRef, Written};
use blitz_dom::{BaseDocument, NodeId, ScrollBehavior};
use dioxus::prelude::MountedData;
use ds::{HostReveal, ScrollSpan, Scrolled, nearest_scroll};

/// The Blitz reveal write, as the `ds::HostReveal` a root provides beside `crate::focus::FOCUS`.
pub const REVEAL: HostReveal = HostReveal(reveal);

/// Show `item` inside `scroller` (both Blitz nodes of one document, the item inside the list).
fn reveal(scroller: &MountedData, item: &MountedData) -> Scrolled {
    let (Some(list), Some(stop)) = (NodeRef::of(scroller), NodeRef::of(item)) else {
        return Scrolled::Unknown;
    };
    let wanted = match list.read(|doc| wanted_offset(doc, list.node, stop.node)) {
        None => return Scrolled::Busy,
        Some(None) => return Scrolled::Unknown,
        Some(Some(wanted)) => wanted,
    };
    let Some((x, y)) = wanted else {
        return Scrolled::Done;
    };
    match list.write(|doc| {
        doc.scroll_to(list.node, x, y, ScrollBehavior::Instant);
        doc.shell_provider.request_redraw();
    }) {
        Written::Done => Scrolled::Done,
        Written::Busy => Scrolled::Busy,
    }
}

/// The list's new scroll offset, `Some(None)` when the item is in view already; `None` when the
/// nodes are gone or the item is not inside the list.
fn wanted_offset(doc: &BaseDocument, list: NodeId, item: NodeId) -> Option<Option<(f64, f64)>> {
    let scroller = doc.get_node(list)?;
    let layout = scroller.final_layout();
    let offset = *scroller.scroll_offset();
    let view = layout.size.height - layout.border.top - layout.border.bottom;
    let top = content_top(doc, item, list)? - layout.border.top;
    let height = doc.get_node(item)?.final_layout().size.height;
    let current = offset.y as f32;
    let span = ScrollSpan {
        start: top,
        length: height,
    };
    let next = nearest_scroll(current, view, span);
    Some(((next - current).abs() >= 0.5).then_some((offset.x, f64::from(next))))
}

/// Where `node`'s border box starts in `ancestor`'s content at no scroll: its layout offsets
/// summed up the layout tree, less the scroll of every scroller between the two.
fn content_top(doc: &BaseDocument, node: NodeId, ancestor: NodeId) -> Option<f32> {
    let mut top = 0.0;
    let mut at = node;
    loop {
        let current = doc.get_node(at)?;
        top += current.final_layout().location.y;
        let parent = current.layout_parent.get()?;
        if parent == ancestor {
            return Some(top);
        }
        top -= doc.get_node(parent)?.scroll_offset().y as f32;
        at = parent;
    }
}
