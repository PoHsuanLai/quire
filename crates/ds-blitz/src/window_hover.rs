//! The window's side of frame link hover: winit's pointer moves, in physical pixels, become the
//! document's logical points and are hit-tested against the frames (`crate::frame_hit`). The
//! host's event hook hears a move before the document does, so the hit uses the layout the user
//! is looking at.

use crate::frame_book::FrameBook;
use crate::frame_hit::link_under;
use crate::frame_hover::{FrameLinkHover, HoverTracker};
use crate::frame_menu::{FrameLinkMenu, ask};
use dioxus_native::winit::event::{ElementState, MouseButton, WindowEvent};
use dioxus_native_dom::NodeHandle;
use ds::prelude::*;

/// The link the window's pointer is on, and where the pointer last was.
pub(crate) struct WindowHover {
    tracker: HoverTracker,
    book: FrameBook,
    last: Point,
}

impl WindowHover {
    /// Tracking against the frames in `book`.
    pub(crate) fn new(book: FrameBook) -> Self {
        WindowHover {
            tracker: HoverTracker::default(),
            book,
            last: Point {
                x: Px(0.0),
                y: Px(0.0),
            },
        }
    }

    /// The crossings `event` makes at `scale` device pixels per logical pixel: a move over the
    /// document's frames, or the pointer leaving the window (off any link). A document the
    /// renderer is holding is skipped; the next move catches up.
    pub(crate) fn crossings(
        &mut self,
        event: &WindowEvent,
        scale: f64,
        document: Option<&NodeHandle>,
    ) -> Vec<FrameLinkHover> {
        let logical = |x: f64, y: f64| Point {
            x: Px((x / scale) as f32),
            y: Px((y / scale) as f32),
        };
        match event {
            WindowEvent::PointerMoved { position, .. } => {
                let at = logical(position.x, position.y);
                self.last = at;
                let Some(doc) = document.and_then(NodeHandle::try_doc) else {
                    return Vec::new();
                };
                let under = link_under(&doc, at);
                drop(doc);
                self.tracker.step(under, at, &self.book)
            }
            WindowEvent::PointerLeft { position, .. } => {
                let at = position.map_or(self.last, |p| logical(p.x, p.y));
                self.tracker.step(None, at, &self.book)
            }
            _ => Vec::new(),
        }
    }

    /// The context menu `event` asks over a link in a frame, if it is the release of the
    /// secondary button (the press Blitz makes a `contextmenu` of) on one.
    pub(crate) fn menu(
        &self,
        event: &WindowEvent,
        scale: f64,
        document: Option<&NodeHandle>,
    ) -> Option<FrameLinkMenu> {
        let WindowEvent::PointerButton {
            state: ElementState::Released,
            position,
            button,
            ..
        } = event
        else {
            return None;
        };
        if button.clone().mouse_button() != Some(MouseButton::Right) {
            return None;
        }
        let at = Point {
            x: Px((position.x / scale) as f32),
            y: Px((position.y / scale) as f32),
        };
        let doc = document.and_then(NodeHandle::try_doc)?;
        let under = link_under(&doc, at);
        drop(doc);
        ask(under, at, &self.book)
    }
}
