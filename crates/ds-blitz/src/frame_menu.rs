//! The link under a context menu inside a frame, for the app's "Copy Link" (mailo's message
//! body). Blitz delivers a frame's `contextmenu` to the `iframe` element and its ancestors, but
//! the app's handler there cannot see inside the frame's document; ds-blitz hit-tests it
//! (`crate::frame_hit`) and tells the app which link, if any, the secondary press ended on. The
//! app hears it just before the page's own `oncontextmenu` for the same press.

use crate::frame_book::FrameBook;
use crate::frame_hit::LinkUnder;
use crate::frame_tag::FrameTag;
use crate::origin::FrameId;
use ds::prelude::*;
use std::fmt;
use std::sync::Arc;

/// A context menu asked over a link inside a frame.
#[derive(Debug, Clone, PartialEq)]
pub struct FrameLinkMenu {
    /// The frame's document.
    pub frame: FrameId,
    /// The frame's `iframe`'s `data-frame-tag`, if it has one.
    pub tag: Option<FrameTag>,
    /// The nearest enclosing `a[href]`'s target, resolved against the frame's base URL.
    pub href: String,
    /// The anchor's text, whitespace-collapsed.
    pub text: String,
    /// The anchor's `title`, if it has one.
    pub title: Option<String>,
    /// Where the pointer is, in the app document's coordinates: where the menu is placed.
    pub at: Point,
}

/// Whether the app hears context menus over links in frames.
#[derive(Clone, Default)]
pub enum FrameMenu {
    /// No: nothing is hit-tested.
    #[default]
    Ignore,
    /// Yes, on the UI thread, with the document free.
    Report(FrameMenuHandler),
}

/// The app's handler for [`FrameMenu::Report`].
#[derive(Clone)]
pub struct FrameMenuHandler(Arc<dyn Fn(FrameLinkMenu) + Send + Sync>);

impl FrameMenuHandler {
    /// Call `handler` with each menu asked over a link.
    pub fn new(handler: impl Fn(FrameLinkMenu) + Send + Sync + 'static) -> Self {
        FrameMenuHandler(Arc::new(handler))
    }
}

impl fmt::Debug for FrameMenu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            FrameMenu::Ignore => "Ignore",
            FrameMenu::Report(_) => "Report(..)",
        })
    }
}

impl fmt::Debug for FrameMenuHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("FrameMenuHandler(..)")
    }
}

/// The menu asked at `at`, over `under`: nothing when the press was not on a link in a frame.
pub fn ask(under: Option<LinkUnder>, at: Point, book: &FrameBook) -> Option<FrameLinkMenu> {
    under.map(|under| FrameLinkMenu {
        tag: book.tag(under.frame),
        frame: under.frame,
        href: under.facts.href,
        text: under.facts.text,
        title: under.facts.title,
        at,
    })
}

/// Hand `menu` to the app, if it listens.
pub fn report(listen: &FrameMenu, menu: Option<FrameLinkMenu>) {
    if let (FrameMenu::Report(FrameMenuHandler(handler)), Some(menu)) = (listen, menu) {
        handler(menu);
    }
}
