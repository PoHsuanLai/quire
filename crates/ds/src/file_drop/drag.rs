//! The values a file drag is made of: what the host hears, what a target sees, what it is
//! handed when the files are let go.

use crate::geometry::Point;
use std::path::PathBuf;

/// A drag of files from outside the window, as one drop target sees it.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum FileDrag {
    /// No files are over this target: nothing is being dragged, the drag is elsewhere in the
    /// window, or it carries no files.
    #[default]
    Idle,
    /// Files are over this target, the pointer at `point` (the window's logical pixels).
    Over {
        /// The dragged files.
        paths: Vec<PathBuf>,
        /// Where the pointer is.
        point: Point,
    },
    /// Files were let go on this target, at `point`. Stays until the next drag enters the
    /// window.
    Dropped {
        /// The dropped files.
        paths: Vec<PathBuf>,
        /// Where they were let go.
        point: Point,
    },
}

/// Files let go on a target: what its `ondrop` hears.
#[derive(Debug, Clone, PartialEq)]
pub struct FileDrop {
    /// The dropped files, in the order the source listed them.
    pub paths: Vec<PathBuf>,
    /// Where they were let go (the window's logical pixels).
    pub point: Point,
}

/// What a drag turned out to carry, once the platform has said.
#[derive(Debug, Clone, PartialEq)]
pub enum Offer {
    /// These files (an empty list carries nothing).
    Files(Vec<PathBuf>),
    /// Something other than files: a URL, text, an image's bytes. Ignored.
    Other,
}

/// One step of a drag from outside the window, as the host hears it from the platform.
#[derive(Debug, Clone, PartialEq)]
pub enum FileDragInput {
    /// A drag came into the window, at `point` if the platform says (Wayland does, X11 only
    /// with the first move).
    Entered {
        /// Where the pointer came in.
        point: Option<Point>,
    },
    /// The platform said what the drag carries. It may come before or after the release.
    Offered(Offer),
    /// The pointer moved to `point` with the drag.
    Moved {
        /// Where the pointer is now.
        point: Point,
    },
    /// The drag was let go over the window.
    Dropped,
    /// The drag left the window, or was cancelled.
    Left,
}

/// Whether the window takes the drag where the pointer is now. The host tells the platform, so
/// the cursor shows a copy or a refusal, and a refused drag let go is not dropped at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DropAcceptance {
    /// Over a target, with files (or not yet known): a copy.
    Copy,
    /// Anywhere else, or not files.
    Refuse,
}

/// Which registered target the host found under a point: the innermost one whose element is
/// the element there or one of its ancestors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DropHit {
    /// The target at this index of the list the host was given.
    Target(usize),
    /// No target is there.
    Nothing,
    /// The document is busy (rendering): keep what was found last.
    Busy,
}
