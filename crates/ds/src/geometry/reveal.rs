//! Scrolling a scroller the least that shows one of its items (sill Q340): a command palette's
//! selected row, cell or header action, moved by a key the caller claimed or by the caller's own
//! `selected`, is brought to the nearest edge of the list, never centred.
//!
//! Blitz's own `scroll_into_view` scrolls the document's viewport only, never the list, so the
//! host does it: ds-native provides [`HostReveal`], which reads the item's place in the list's
//! layout and sets the list's scroll offset. Without one (a webview) the item's own
//! `scrollIntoView` with the nearest block is used, which is the same rule.

use crate::geometry::measure::{BUSY_ATTEMPTS, laid_out_rect};
use crate::guarded::guarded_call;
use crate::time::{FRAME_SLACK, sleep};
use dioxus::html::{ScrollBehavior, ScrollLogicalPosition, ScrollToOptions};
use dioxus::prelude::*;

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

/// The host's reveal write, provided as root context by ds-native (`launch`, its harness and
/// `ds_native::focus::provide`): scroll the scroller `.0`'s own content the least that shows the
/// item `.1` inside it, with no animation.
#[derive(Debug, Clone, Copy)]
pub struct HostReveal(pub fn(&MountedData, &MountedData) -> Scrolled);

/// An item's extent along the scroll axis, in the scroller's content coordinates (0 is the top of
/// its content at no scroll), and the scroller's.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollSpan {
    /// Where it starts.
    pub start: f32,
    /// How long it is.
    pub length: f32,
}

/// The scroll offset that shows `item` in a scrollport `view` long scrolled to `current`, moving
/// the least: unchanged when the item is inside the view (or the view inside it), else the item's
/// nearer edge aligned with the view's (CSSOM `scrollIntoView` with `block: nearest`).
pub fn nearest_scroll(current: f32, view: f32, item: ScrollSpan) -> f32 {
    let end = item.start + item.length;
    let inside = item.start >= current && end <= current + view;
    let covers = item.start <= current && end >= current + view;
    if inside || covers {
        return current;
    }
    let to_start = item.start;
    let to_end = end - view;
    if (to_start - current).abs() < (to_end - current).abs() {
        to_start
    } else {
        to_end
    }
}

/// Show `item` inside `scroller`, once the item has been laid out: through the host's
/// [`HostReveal`], else the renderer's own nearest-edge `scrollIntoView`. Call it from a task.
pub(crate) async fn reveal(scroller: &MountedData, item: &MountedData) -> Scrolled {
    if laid_out_rect(item).await.is_none() {
        return Scrolled::Unknown;
    }
    match try_consume_context::<HostReveal>() {
        Some(HostReveal(write)) => {
            for _ in 0..BUSY_ATTEMPTS {
                match write(scroller, item) {
                    Scrolled::Busy => sleep(FRAME_SLACK).await,
                    done => return done,
                }
            }
            Scrolled::Busy
        }
        None => unhosted(item).await,
    }
}

/// The renderer's own nearest-edge scroll, guarded as a focus write is (`crate::guarded`).
async fn unhosted(item: &MountedData) -> Scrolled {
    let options = ScrollToOptions {
        behavior: ScrollBehavior::Instant,
        vertical: ScrollLogicalPosition::Nearest,
        horizontal: ScrollLogicalPosition::Nearest,
    };
    match guarded_call(|| item.scroll_to_with_options(options)).await {
        Some(Ok(())) => Scrolled::Done,
        Some(Err(_)) => Scrolled::Unknown,
        None => Scrolled::Busy,
    }
}

#[cfg(test)]
mod tests {
    use super::{ScrollSpan, nearest_scroll};

    #[test]
    fn the_scroll_moves_the_least_that_shows_the_item() {
        // (current, view, item start, item length, wanted), a 100 long view.
        const CASES: &[(&str, f32, f32, f32, f32, f32)] = &[
            ("inside stays", 0.0, 100.0, 10.0, 20.0, 0.0),
            (
                "flush with both edges stays",
                50.0,
                100.0,
                50.0,
                100.0,
                50.0,
            ),
            ("below aligns its bottom", 0.0, 100.0, 150.0, 20.0, 70.0),
            ("half below aligns its bottom", 0.0, 100.0, 90.0, 20.0, 10.0),
            ("above aligns its top", 200.0, 100.0, 40.0, 20.0, 40.0),
            (
                "half above aligns its top",
                200.0,
                100.0,
                190.0,
                20.0,
                190.0,
            ),
            (
                "taller than the view, covering it, stays",
                100.0,
                100.0,
                50.0,
                300.0,
                100.0,
            ),
            (
                "taller than the view, below, aligns the nearer edge",
                0.0,
                100.0,
                150.0,
                300.0,
                150.0,
            ),
        ];
        for &(name, current, view, start, length, want) in CASES {
            let got = nearest_scroll(current, view, ScrollSpan { start, length });
            assert!((got - want).abs() < 0.001, "{name}: {got}");
        }
    }
}
