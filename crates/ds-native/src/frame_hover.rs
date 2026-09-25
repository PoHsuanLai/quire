//! The pointer coming onto and leaving a link inside a frame, for the app's link pill (mailo
//! shows where a link goes before it is clicked). Reported once per change, never per move: the
//! tracker remembers the link it last reported and speaks only when that changes.

use crate::frame_book::FrameBook;
use crate::frame_hit::LinkUnder;
use crate::frame_tag::FrameTag;
use crate::origin::FrameId;
use ds::Point;
use std::fmt;
use std::sync::Arc;

/// The pointer came onto a link in a frame, or left it.
#[derive(Debug, Clone, PartialEq)]
pub struct FrameLinkHover {
    /// The frame's document.
    pub frame: FrameId,
    /// The frame's `iframe`'s `data-frame-tag`, if it has one.
    pub tag: Option<FrameTag>,
    /// The link's target, resolved against the frame's base URL.
    pub href: String,
    /// The anchor's text, whitespace-collapsed.
    pub text: String,
    /// The anchor's `title`, if it has one.
    pub title: Option<String>,
    /// Where the pointer is, in the app document's coordinates: where a pill is placed.
    pub at: Point,
    /// Onto the link or off it.
    pub phase: HoverPhase,
}

/// Which way the pointer crossed a link's edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HoverPhase {
    /// It came onto the link.
    Enter,
    /// It left the link (onto another, onto nothing, or out of the frame).
    Leave,
}

/// Whether the app hears the pointer crossing links in frames.
#[derive(Clone, Default)]
pub enum FrameHover {
    /// No: nothing is hit-tested.
    #[default]
    Ignore,
    /// Yes, on the UI thread, with the document free.
    Report(FrameHoverHandler),
}

/// The app's handler for [`FrameHover::Report`].
#[derive(Clone)]
pub struct FrameHoverHandler(Arc<dyn Fn(FrameLinkHover) + Send + Sync>);

impl FrameHoverHandler {
    /// Call `handler` with each crossing.
    pub fn new(handler: impl Fn(FrameLinkHover) + Send + Sync + 'static) -> Self {
        FrameHoverHandler(Arc::new(handler))
    }
}

impl fmt::Debug for FrameHover {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            FrameHover::Ignore => "Ignore",
            FrameHover::Report(_) => "Report(..)",
        })
    }
}

impl fmt::Debug for FrameHoverHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("FrameHoverHandler(..)")
    }
}

/// The link the pointer was last reported on, if any.
#[derive(Debug, Default)]
pub(crate) struct HoverTracker {
    over: Option<(LinkUnder, Option<FrameTag>)>,
}

impl HoverTracker {
    /// The pointer is at `at`, over `under`: the crossings since the last move, a leave before
    /// an enter. Nothing while it stays on the same link or off every link.
    pub(crate) fn step(
        &mut self,
        under: Option<LinkUnder>,
        at: Point,
        book: &FrameBook,
    ) -> Vec<FrameLinkHover> {
        let same = |(was, _): &(LinkUnder, Option<FrameTag>)| {
            under
                .as_ref()
                .is_some_and(|now| (now.frame, now.anchor) == (was.frame, was.anchor))
        };
        if self.over.as_ref().is_some_and(same) {
            return Vec::new();
        }
        let left = self.over.take();
        self.over = under.map(|now| {
            let tag = book.tag(now.frame);
            (now, tag)
        });
        let leave = left.map(|over| crossing(over, at, HoverPhase::Leave));
        let enter = self
            .over
            .clone()
            .map(|over| crossing(over, at, HoverPhase::Enter));
        leave.into_iter().chain(enter).collect()
    }
}

/// The crossing of `over`'s link at `at`.
fn crossing(
    (under, tag): (LinkUnder, Option<FrameTag>),
    at: Point,
    phase: HoverPhase,
) -> FrameLinkHover {
    FrameLinkHover {
        frame: under.frame,
        tag,
        href: under.facts.href,
        text: under.facts.text,
        title: under.facts.title,
        at,
        phase,
    }
}

/// Hand each of `crossings` to the app, if it listens.
pub(crate) fn report(hover: &FrameHover, crossings: Vec<FrameLinkHover>) {
    if let FrameHover::Report(FrameHoverHandler(handler)) = hover {
        crossings.into_iter().for_each(|crossing| handler(crossing));
    }
}
