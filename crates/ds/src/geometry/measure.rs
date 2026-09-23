//! Measuring a mounted element, the one layout read the design system does.
//!
//! Two phases (spike S9): `get_client_rect` inside `onmounted` returns 0 x 0, and is right only
//! after the next resolve. So the `onmounted` handler only keeps the element; the rect is read
//! on the following frame, never inside the handler.

use super::units::{Point, Px, Rect, Size};
use crate::time::{FRAME_SLACK, sleep};
use dioxus::html::geometry::PixelsRect;
use dioxus::prelude::*;
use std::rc::Rc;

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
pub fn use_rect() -> RectProbe {
    RectProbe {
        rect: use_signal(|| None),
        mounted: use_signal(|| None),
    }
}

/// How many frames a read waits for layout before giving up on a zero rect.
const READ_ATTEMPTS: usize = 3;

/// The element's rect once layout has run: wait a frame, read, and read again a frame later
/// while the renderer still answers 0 x 0 (spike S9). `None` when the renderer cannot measure.
async fn read_after_layout(element: &MountedRef) -> Option<Rect> {
    let mut last = None;
    for _ in 0..READ_ATTEMPTS {
        sleep(FRAME_SLACK).await;
        let read = element.0.get_client_rect().await.ok().map(from_pixels)?;
        if read.size.width.0 > 0.0 || read.size.height.0 > 0.0 {
            return Some(read);
        }
        last = Some(read);
    }
    last
}

/// A renderer rect in logical pixels.
fn from_pixels(rect: PixelsRect) -> Rect {
    Rect {
        origin: Point {
            x: Px(rect.origin.x as f32),
            y: Px(rect.origin.y as f32),
        },
        size: Size {
            width: Px(rect.size.width as f32),
            height: Px(rect.size.height as f32),
        },
    }
}
