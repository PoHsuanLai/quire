//! What a link clicked inside a frame does (mailo Phase B, G7). Blitz reloads the frame with the
//! link's target (its `IframeNavigationProvider` sends the parent a `NavigateIframe`), which for
//! a mail body means fetching a stranger's page into the reader. ds-native gives every frame
//! document its own navigation provider instead (through `crate::frames`, as the document is
//! parsed): the frame never navigates, and the app either hears the click or nothing happens.
//!
//! A click is delivered through a channel, not called from the provider: Blitz calls it while
//! the document is held, and the app's handler must be free to touch the document (focus,
//! measure) without a "RefCell already borrowed" (sill FINDINGS Q43). The link's text and title
//! are read as it is delivered, from the frame's document, then free: the provider hears only the
//! URL.

use crate::frame_anchor::LinkFacts;
use crate::frame_book::FrameBook;
use crate::frame_hover::{FrameHover, FrameHoverHandler, FrameLinkHover};
use crate::frame_tag::FrameTag;
use crate::origin::FrameId;
use blitz_traits::navigation::{NavigationOptions, NavigationProvider};
use std::fmt;
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

/// A link clicked inside a frame.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FrameLink {
    /// The frame's document.
    pub frame: FrameId,
    /// The frame's `iframe`'s `data-frame-tag`: which of the app's frames it is (mailo's
    /// message), if the element has one.
    pub tag: Option<FrameTag>,
    /// The link's target, resolved against the frame's base URL.
    pub href: String,
    /// The anchor's text content, whitespace-collapsed: what the reader saw, to compare with
    /// `href` (mailo's link honesty check). Empty if the anchor could not be found again.
    pub text: String,
    /// The anchor's `title`, if it has one.
    pub title: Option<String>,
}

/// What a link clicked inside a frame does. The frame never navigates either way.
#[derive(Clone, Default)]
pub enum FrameLinks {
    /// Nothing: the click is dropped.
    #[default]
    Inert,
    /// The app hears it (mailo opens it in the browser), on the UI thread, with the document
    /// free; and, if `hover` reports, hears the pointer come onto and leave each link.
    Intercept {
        /// Called with each click.
        click: FrameLinkHandler,
        /// Whether the pointer crossing a link is reported too.
        hover: FrameHover,
    },
}

impl FrameLinks {
    /// Call `handler` with every link clicked inside a frame.
    pub fn intercept(handler: impl Fn(FrameLink) + Send + Sync + 'static) -> Self {
        FrameLinks::Intercept {
            click: FrameLinkHandler(Arc::new(handler)),
            hover: FrameHover::Ignore,
        }
    }

    /// Also call `handler` as the pointer comes onto and leaves a link inside a frame (a link
    /// pill). `Inert` stays inert: a link that does nothing has nothing to preview.
    pub fn with_hover(self, handler: impl Fn(FrameLinkHover) + Send + Sync + 'static) -> Self {
        match self {
            FrameLinks::Inert => FrameLinks::Inert,
            FrameLinks::Intercept { click, .. } => FrameLinks::Intercept {
                click,
                hover: FrameHover::Report(FrameHoverHandler::new(handler)),
            },
        }
    }

    /// Whether the pointer crossing a link is reported.
    pub(crate) fn hover(&self) -> FrameHover {
        match self {
            FrameLinks::Inert => FrameHover::Ignore,
            FrameLinks::Intercept { hover, .. } => hover.clone(),
        }
    }
}

impl fmt::Debug for FrameLinks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameLinks::Inert => f.write_str("Inert"),
            FrameLinks::Intercept { hover, .. } => f
                .debug_struct("Intercept")
                .field("hover", hover)
                .finish_non_exhaustive(),
        }
    }
}

impl fmt::Debug for FrameLinkHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("FrameLinkHandler(..)")
    }
}

/// The app's handler for [`FrameLinks::Intercept`].
#[derive(Clone)]
pub struct FrameLinkHandler(Arc<dyn Fn(FrameLink) + Send + Sync>);

/// A click as the frame's navigation provider hears it, before the app is told.
#[derive(Debug)]
struct Clicked {
    frame: FrameId,
    href: String,
}

/// Every frame document's navigation provider: a click goes on the channel, or nowhere when the
/// app keeps frame links inert.
struct FrameNav {
    sender: Option<UnboundedSender<Clicked>>,
}

impl NavigationProvider for FrameNav {
    fn navigate_to(&self, options: NavigationOptions) {
        if let Some(sender) = &self.sender {
            let _ = sender.send(Clicked {
                frame: FrameId::of(options.source_document),
                href: options.url.to_string(),
            });
        }
    }
}

/// Where the clicks a document's frames report are delivered to the app.
pub(crate) struct LinkInbox {
    receiver: UnboundedReceiver<Clicked>,
    handler: Option<FrameLinkHandler>,
    /// Where the clicked frame's tag is read.
    book: FrameBook,
}

impl LinkInbox {
    /// Hand every click waiting to the app, now: the headless document calls it each frame.
    /// `read` finds what the link says; it must release the document before it returns.
    pub(crate) fn drain(&mut self, read: &dyn Fn(FrameId, &str) -> LinkFacts) {
        while let Ok(link) = self.receiver.try_recv() {
            self.deliver(link, read);
        }
    }

    /// Hand each click to the app as it arrives, for the document's life: the window's task.
    pub(crate) async fn serve(mut self, read: impl Fn(FrameId, &str) -> LinkFacts) {
        while let Some(link) = self.receiver.recv().await {
            self.deliver(link, &read);
        }
    }

    fn deliver(&self, clicked: Clicked, read: &dyn Fn(FrameId, &str) -> LinkFacts) {
        if let Some(FrameLinkHandler(handler)) = &self.handler {
            let facts = read(clicked.frame, &clicked.href);
            handler(FrameLink {
                frame: clicked.frame,
                tag: self.book.tag(clicked.frame),
                href: clicked.href,
                text: facts.text,
                title: facts.title,
            });
        }
    }
}

/// The frame documents' navigation provider for `links`, and the inbox its clicks arrive in,
/// tagged from `book`.
pub(crate) fn frame_links(
    links: &FrameLinks,
    book: FrameBook,
) -> (Arc<dyn NavigationProvider>, LinkInbox) {
    let (sender, receiver) = unbounded_channel();
    let (sender, handler) = match links {
        FrameLinks::Inert => (None, None),
        FrameLinks::Intercept { click, .. } => (Some(sender), Some(click.clone())),
    };
    (
        Arc::new(FrameNav { sender }),
        LinkInbox {
            receiver,
            handler,
            book,
        },
    )
}

/// What the link clicked toward `href` in `frame` says, read from `top`'s frames.
pub(crate) fn read_link(top: &blitz_dom::BaseDocument, frame: FrameId, href: &str) -> LinkFacts {
    crate::frame_tree::in_frame(top, frame, &|doc| crate::frame_anchor::clicked(doc, href))
        .unwrap_or_else(|| LinkFacts::bare(href))
}
