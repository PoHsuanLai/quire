//! What a link clicked inside a frame does (mailo Phase B, G7). Blitz reloads the frame with the
//! link's target (its `IframeNavigationProvider` sends the parent a `NavigateIframe`), which for
//! a mail body means fetching a stranger's page into the reader. ds-native gives every frame
//! document its own navigation provider instead (through `crate::frames`, as the document is
//! parsed): the frame never navigates, and the app either hears the click or nothing happens.
//!
//! A click is delivered through a channel, not called from the provider: Blitz calls it while
//! the document is held, and the app's handler must be free to touch the document (focus,
//! measure) without a "RefCell already borrowed" (sill FINDINGS Q43).

use crate::frame_book::FrameBook;
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
}

/// What a link clicked inside a frame does. The frame never navigates either way.
#[derive(Clone, Default)]
pub enum FrameLinks {
    /// Nothing: the click is dropped.
    #[default]
    Inert,
    /// The app hears it (mailo opens it in the browser), on the UI thread, with the document
    /// free.
    Intercept(FrameLinkHandler),
}

impl FrameLinks {
    /// Call `handler` with every link clicked inside a frame.
    pub fn intercept(handler: impl Fn(FrameLink) + Send + Sync + 'static) -> Self {
        FrameLinks::Intercept(FrameLinkHandler(Arc::new(handler)))
    }
}

impl fmt::Debug for FrameLinks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            FrameLinks::Inert => "Inert",
            FrameLinks::Intercept(_) => "Intercept(..)",
        })
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
    pub(crate) fn drain(&mut self) {
        while let Ok(link) = self.receiver.try_recv() {
            self.deliver(link);
        }
    }

    /// Hand each click to the app as it arrives, for the document's life: the window's task.
    pub(crate) async fn serve(mut self) {
        while let Some(link) = self.receiver.recv().await {
            self.deliver(link);
        }
    }

    fn deliver(&self, clicked: Clicked) {
        if let Some(FrameLinkHandler(handler)) = &self.handler {
            handler(FrameLink {
                frame: clicked.frame,
                tag: self.book.tag(clicked.frame),
                href: clicked.href,
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
        FrameLinks::Intercept(handler) => (Some(sender), Some(handler.clone())),
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
