//! The image under a click inside a frame, for the app's look at it (mailo opens an inline
//! image in Peek). A click on an `img` in a frame document does nothing, and the app's handlers
//! around the `iframe` cannot see inside it; ds-blitz hit-tests the primary release
//! (`crate::frame_hit`) and tells the app which image, if any, it ended on. An image inside a link
//! is the link's click (`crate::frame_links`), never reported here.

use crate::frame_book::FrameBook;
use crate::frame_hit::ImageUnder;
use crate::frame_tag::FrameTag;
use crate::origin::FrameId;
use ds::prelude::*;
use std::fmt;
use std::sync::Arc;

/// An image clicked inside a frame.
#[derive(Debug, Clone, PartialEq)]
pub struct FrameImageClick {
    /// The frame's document.
    pub frame: FrameId,
    /// The frame's `iframe`'s `data-frame-tag`, if it has one.
    pub tag: Option<FrameTag>,
    /// The image's `src`, resolved against the frame's base URL: a `cid:` reference stays
    /// `cid:`, so the app finds the message part it names.
    pub src: String,
    /// The image's `alt`, if it has one.
    pub alt: Option<String>,
    /// Where the pointer was released, in the app document's coordinates.
    pub at: Point,
}

/// Whether the app hears clicks on images in frames.
#[derive(Clone, Default)]
pub enum FrameImages {
    /// No: nothing is hit-tested.
    #[default]
    Ignore,
    /// Yes, on the UI thread, with the document free.
    Report(FrameImageHandler),
}

/// The app's handler for [`FrameImages::Report`].
#[derive(Clone)]
pub struct FrameImageHandler(Arc<dyn Fn(FrameImageClick) + Send + Sync>);

impl FrameImageHandler {
    /// Call `handler` with each click on an image.
    pub fn new(handler: impl Fn(FrameImageClick) + Send + Sync + 'static) -> Self {
        FrameImageHandler(Arc::new(handler))
    }
}

impl fmt::Debug for FrameImages {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            FrameImages::Ignore => "Ignore",
            FrameImages::Report(_) => "Report(..)",
        })
    }
}

impl fmt::Debug for FrameImageHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("FrameImageHandler(..)")
    }
}

/// The click released at `at`, over `under`: nothing when it was not on an image in a frame.
pub fn ask(under: Option<ImageUnder>, at: Point, book: &FrameBook) -> Option<FrameImageClick> {
    under.map(|under| FrameImageClick {
        tag: book.tag(under.frame),
        frame: under.frame,
        src: under.facts.src,
        alt: under.facts.alt,
        at,
    })
}

/// Hand `click` to the app, if it listens.
pub fn report(listen: &FrameImages, click: Option<FrameImageClick>) {
    if let (FrameImages::Report(FrameImageHandler(handler)), Some(click)) = (listen, click) {
        handler(click);
    }
}
