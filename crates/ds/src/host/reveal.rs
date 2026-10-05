//! Scrolling a scroller the least that shows one of its items: a command palette's
//! selected row, cell or header action, moved by a key the caller claimed or by the caller's own
//! `selected`, is brought to the nearest edge of the list, never centred.
//!
//! Blitz's own `scroll_into_view` scrolls the document's viewport only, never the list, so the
//! host does it: [`GeometryHost::reveal`](crate::host::parts::GeometryHost::reveal) reads the item's place in
//! the list's layout and sets the list's scroll offset.

use crate::host::document::use_document_host;
use crate::host::measure::{BUSY_ATTEMPTS, laid_out_rect};
use dioxus::prelude::*;
use ds_style::busy::wait_out_busy;

pub use ds_core::geometry::scroll::{ScrollSpan, nearest_scroll};

/// One attempt at scrolling a scroller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scrolled {
    /// The item is in view (the scroller moved, or it already was).
    Done,
    /// The document is busy (rendering); try again next frame.
    Busy,
    /// The host cannot scroll these elements (not its nodes, gone, or not nested).
    Unknown,
}

/// Show `item` inside `scroller`, once the item has been laid out, through the host. Call it from
/// a task.
pub async fn reveal(scroller: &MountedData, item: &MountedData) -> Scrolled {
    if laid_out_rect(item).await.is_none() {
        return Scrolled::Unknown;
    }
    let host = use_document_host();
    for attempt in 0..BUSY_ATTEMPTS {
        match host.geometry().reveal(scroller, item) {
            Scrolled::Busy => wait_out_busy(attempt).await,
            done => return done,
        }
    }
    Scrolled::Busy
}
