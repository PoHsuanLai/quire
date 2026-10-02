//! Measuring a mounted element, the one layout read the design system does.
//!
//! Two phases (spike S9): `get_client_rect` inside `onmounted` returns 0 x 0, and is right only
//! after the next resolve. So the `onmounted` handler only keeps the element; the rect is read
//! on the following frame, never inside the handler.
//!
//! Every read goes through [`client_rect`]. On dioxus-native, `get_client_rect` borrows the
//! document mutably, and a task woken in the same turn as a dirty scope is polled inside
//! `render_immediate` while the renderer already holds that borrow: the read would panic
//! ("RefCell already borrowed"). The host's [`GeometryHost::measure`](crate::host::parts::GeometryHost::measure)
//! answers [`Measured::Busy`] instead, and the read waits a frame.

use crate::host::document::use_document_host;
use crate::host::resized::WindowResized;
use dioxus::prelude::*;
use ds_core::geometry::units::{Point, Rect};
use ds_core::time::{FRAME_SLACK, clock::sleep};
use ds_style::busy::{after_render, wait_out_busy};
use std::rc::Rc;

/// One attempt at reading an element's rect through the host.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Measured {
    /// The element's border box, in logical pixels.
    At(Rect),
    /// The document is busy (rendering); ask again next frame.
    Busy,
    /// The host cannot measure this element (not its node, or gone).
    Unknown,
}

/// How many frames a read waits for a busy document before giving up.
pub(crate) const BUSY_ATTEMPTS: usize = 8;

/// `element`'s rect now, through the host. `None` when the host cannot measure it. Call it from a
/// task, never from inside a handler or render.
pub async fn client_rect(element: &MountedData) -> Option<Rect> {
    let host = use_document_host();
    for attempt in 0..BUSY_ATTEMPTS {
        match host.geometry().measure(element) {
            Measured::At(rect) => return Some(rect),
            Measured::Busy => wait_out_busy(attempt).await,
            Measured::Unknown => return None,
        }
    }
    None
}

/// A mounted element, kept so its rect can be read again when the surface moves.
#[derive(Clone)]
pub struct MountedRef(pub Rc<MountedData>);

impl PartialEq for MountedRef {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl std::fmt::Debug for MountedRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MountedRef")
    }
}

/// What a floating surface is placed against.
#[derive(Debug, Clone, PartialEq)]
pub enum Anchor {
    /// A point: the caret, a right-click.
    Point(Point),
    /// A rect already known: a button's, a selection's.
    Rect(Rect),
    /// A mounted element, measured when placing.
    Mounted(MountedRef),
}

/// The rect of one mounted element, updated from its `onmounted` event.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectProbe {
    rect: Signal<Option<Rect>>,
    mounted: Signal<Option<MountedRef>>,
}

impl RectProbe {
    /// The last measured rect, or `None` before the element mounted.
    pub fn rect(&self) -> Option<Rect> {
        (self.rect)()
    }

    /// Hand this the element's `onmounted` event. It keeps the element and schedules the read
    /// for the next frame; it does not measure.
    pub fn on_mounted(&self, event: Event<MountedData>) {
        let element = MountedRef(event.data());
        let mut mounted = self.mounted;
        let mut rect = self.rect;
        mounted.set(Some(element.clone()));
        spawn(async move {
            if let Some(read) = read_after_layout(&element).await {
                rect.set(Some(read));
            }
        });
    }

    /// The element as an anchor, once mounted.
    pub fn anchor(&self) -> Option<Anchor> {
        (self.mounted)().map(Anchor::Mounted)
    }
}

/// A probe for one element's rect.
///
/// The rect is read again after each change of the window's size or scale (`WindowResized`, where
/// the host provides it), so an element that follows the window reports where it is now.
pub fn use_rect() -> RectProbe {
    let probe = RectProbe {
        rect: use_signal(|| None),
        mounted: use_signal(|| None),
    };
    let resized = try_consume_context::<WindowResized>();
    use_effect(move || {
        let changes = resized.map_or(0, |resized| resized.count());
        let Some(element) = (probe.mounted)() else {
            return;
        };
        if changes == 0 {
            return;
        }
        let mut rect = probe.rect;
        spawn(async move {
            // The window's layout follows its resize by a frame; wait it out before reading.
            sleep(FRAME_SLACK).await;
            if let Some(read) = read_after_layout(&element).await {
                rect.set(Some(read));
            }
        });
    });
    probe
}

/// How many frames a read waits for layout before giving up on a zero rect.
const READ_ATTEMPTS: usize = 3;

/// The element's rect once layout has run: wait a frame, read, and read again a frame later
/// while the renderer still answers 0 x 0 (spike S9). `None` when the renderer cannot measure.
async fn read_after_layout(element: &MountedRef) -> Option<Rect> {
    let mut last = None;
    for _ in 0..READ_ATTEMPTS {
        sleep(FRAME_SLACK).await;
        let read = client_rect(&element.0).await?;
        if read.size.width.0 > 0.0 || read.size.height.0 > 0.0 {
            return Some(read);
        }
        last = Some(read);
    }
    last
}

/// When a read that found no layout yet is tried again: every frame at first (a freshly mapped
/// surface lays out within a few), then every [`SLOW_RETRY`], then not at all.
pub(crate) fn layout_retry(attempt: usize) -> Option<std::time::Duration> {
    match attempt {
        0..FAST_RETRIES => Some(FRAME_SLACK),
        FAST_RETRIES..LAYOUT_RETRIES => Some(SLOW_RETRY),
        _ => None,
    }
}

/// Reads tried a frame apart before slowing down (about half a second).
const FAST_RETRIES: usize = 30;
/// Reads tried in all before giving up (about three seconds more at [`SLOW_RETRY`]).
const LAYOUT_RETRIES: usize = 60;
/// The wait between reads once the frame-paced ones found no layout.
const SLOW_RETRY: std::time::Duration = std::time::Duration::from_millis(100);

/// Whether `rect` has an area: a rect read before layout is 0 x 0 at the origin.
pub(crate) fn laid_out(rect: Rect) -> bool {
    rect.size.width.0 > 0.0 && rect.size.height.0 > 0.0
}

/// `element`'s rect once it has been laid out: read a frame from now and again on
/// [`layout_retry`]'s schedule while it is still empty (a surface mapped again reads its rows
/// before its first layout). `None` when the renderer cannot measure it or it
/// never gains an area; never an empty rect.
pub(crate) async fn laid_out_rect(element: &MountedData) -> Option<Rect> {
    sleep(FRAME_SLACK).await;
    for attempt in 0.. {
        let read = client_rect(element).await?;
        if laid_out(read) {
            return Some(read);
        }
        sleep(layout_retry(attempt)?).await;
    }
    None
}

/// `element`'s rect as the last layout left it, if it has an area: a row that was already on
/// screen, read in the frame that asked, once the render that asked has ended (so after that
/// render's own effects, such as a palette's selection report). `None` for an element not laid
/// out yet.
pub(crate) async fn laid_out_now(element: &MountedData) -> Option<Rect> {
    after_render().await;
    client_rect(element).await.filter(|read| laid_out(*read))
}

/// How many frames running an element's rect must be unchanged before [`follow_rect`] takes it
/// as settled.
const STILL_FRAMES: u8 = 4;

/// The most frames [`follow_rect`] follows an element for.
const FOLLOW_FRAMES: u16 = 240;

/// Follow `element`'s client rect frame by frame until it has held still for a few frames
/// running, calling `placed` with each new one: layout may not have reached the element at first
/// (a rect of no size, or none) and may move it after (fonts, siblings arriving), so the first
/// reading is not the answer. This is the one "settle until stable" for every floating surface
/// that places itself against an element (popovers, tooltips, hover cards, dock labels); a
/// surface placed against the latest `placed` never stands at the origin or over its anchor.
pub async fn follow_rect(element: &MountedData, mut placed: impl FnMut(Rect)) {
    let mut last = None;
    let mut still = 0u8;
    for _ in 0..FOLLOW_FRAMES {
        sleep(FRAME_SLACK).await;
        let Some(rect) = client_rect(element).await else {
            continue;
        };
        if rect.size.width.0 <= 0.0 || rect.size.height.0 <= 0.0 {
            continue;
        }
        if last == Some(rect) {
            still += 1;
            if still >= STILL_FRAMES {
                return;
            }
        } else {
            still = 0;
            last = Some(rect);
            placed(rect);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SLOW_RETRY, laid_out, layout_retry};
    use ds_core::geometry::units::{Point, Px, Rect, Size};
    use ds_core::time::FRAME_SLACK;

    #[test]
    fn a_read_before_layout_is_tried_again_frame_paced_then_slower_then_not() {
        // (attempt, the wait before the next read)
        let cases = [
            (0, Some(FRAME_SLACK)),
            (29, Some(FRAME_SLACK)),
            (30, Some(SLOW_RETRY)),
            (59, Some(SLOW_RETRY)),
            (60, None),
        ];
        for (attempt, want) in cases {
            assert_eq!(layout_retry(attempt), want, "attempt {attempt}");
        }
    }

    #[test]
    fn only_a_rect_with_an_area_is_laid_out() {
        let at = |width: f32, height: f32| Rect {
            origin: Point::default(),
            size: Size {
                width: Px(width),
                height: Px(height),
            },
        };
        let cases = [
            (at(0.0, 0.0), false),
            (at(668.0, 0.0), false),
            (at(0.0, 45.0), false),
            (at(668.0, 45.1), true),
        ];
        for (rect, want) in cases {
            assert_eq!(laid_out(rect), want, "{rect:?}");
        }
    }
}
