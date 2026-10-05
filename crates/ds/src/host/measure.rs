//! Measuring a mounted element: the rect a component keeps ([`use_rect`], fed by the host's frame
//! phase through [`use_layout`](crate::host::layout::use_layout)) and the one-shot async read
//! ([`client_rect`]) that a host with no phase, and a few waits that have not moved onto the
//! phase, use.
//!
//! Two phases (spike S9): `get_client_rect` inside `onmounted` returns 0 x 0, and is right only
//! after the next resolve. So the `onmounted` handler only keeps the element; the rect is
//! published after layout (or, with no phase, read on the following frame), never inside the
//! handler.
//!
//! Every one-shot read goes through [`client_rect`]. On dioxus-native, `get_client_rect` borrows the
//! document mutably, and a task woken in the same turn as a dirty scope is polled inside
//! `render_immediate` while the renderer already holds that borrow: the read would panic
//! ("RefCell already borrowed"). The host's [`GeometryHost::measure`](crate::host::parts::GeometryHost::measure)
//! answers [`Measured::Busy`] instead, and the read waits a frame.

use crate::host::document::use_document_host;
use crate::host::layout::use_layout_into;
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
    /// The last rect the host published, or `None` before the element mounted and was laid out.
    pub fn rect(&self) -> Option<Rect> {
        (self.rect)()
    }

    /// Hand this the element's `onmounted` event. It keeps the element; the host publishes its
    /// rect from then on ([`use_layout`](crate::host::layout::use_layout)).
    pub fn on_mounted(&self, event: Event<MountedData>) {
        let mut mounted = self.mounted;
        mounted.set(Some(MountedRef(event.data())));
    }

    /// Read the element's rect again, now: layout may have moved it since its first read (a
    /// surface placed against an anchor read later than it). Call it from a task.
    pub(crate) async fn reread(&self) {
        let Some(element) = (self.mounted)() else {
            return;
        };
        if let Some(read) = client_rect(&element.0).await.filter(|read| laid_out(*read)) {
            let _ = ds_style::task::try_set(self.rect, Some(read));
        }
    }

    /// The element as an anchor, once mounted.
    pub fn anchor(&self) -> Option<Anchor> {
        (self.mounted)().map(Anchor::Mounted)
    }
}

/// A probe for one element's rect, kept by the host's frame phase: it follows the element through
/// every layout that moves or resizes it, the window's resize included.
pub fn use_rect() -> RectProbe {
    let probe = RectProbe {
        rect: use_signal(|| None),
        mounted: use_signal(|| None),
    };
    use_layout_into(probe.mounted.into(), probe.rect);
    probe
}

/// How many frames a read waits for layout before giving up on a zero rect.
const READ_ATTEMPTS: usize = 3;

/// The element's rect once layout has run: wait a frame, read, and read again a frame later
/// while the renderer still answers 0 x 0 (spike S9). `None` when the renderer cannot measure.
pub(crate) async fn read_after_layout(element: &MountedRef) -> Option<Rect> {
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

/// Follow `first`'s and `second`'s client rects together, frame by frame, until both have held
/// still for a few frames: [`follow_rect`]'s settle for a pair, in one wait rather than two
/// (a submenu measures its parent row and its panel). The rects read last, or `None` when `first`
/// never had an area; `second` is `None` when it is absent or never had one.
pub async fn follow_rects(
    first: &MountedData,
    second: Option<&MountedData>,
) -> Option<(Rect, Option<Rect>)> {
    let mut last = None;
    let mut still = 0u8;
    for _ in 0..FOLLOW_FRAMES {
        sleep(FRAME_SLACK).await;
        let Some(rect) = client_rect(first).await.filter(|rect| laid_out(*rect)) else {
            continue;
        };
        let other = match second {
            Some(element) => client_rect(element).await.filter(|rect| laid_out(*rect)),
            None => None,
        };
        let read = Some((rect, other));
        if last == read {
            still += 1;
            if still >= STILL_FRAMES {
                break;
            }
        } else {
            still = 0;
            last = read;
        }
    }
    last
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
