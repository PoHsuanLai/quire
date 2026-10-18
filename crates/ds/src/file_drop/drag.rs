//! The values a file drag is made of: what the host hears, what a target sees, what it is
//! handed when the files are let go.

use ds_core::geometry::units::Point;
use std::path::PathBuf;

/// A drag of files from outside the window, as one drop target sees it.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum FileDrag {
    /// No files are over this target: nothing is being dragged, the drag is elsewhere in the
    /// window, or it carries no files.
    #[default]
    Idle,
    /// Files are over this target, the pointer at the files' `point` (the window's logical
    /// pixels).
    Over(FileDrop),
    /// Files were let go on this target, at their `point`. Stays until the next drag enters the
    /// window.
    Dropped(FileDrop),
}

/// Files over a target, or let go on it (what its `ondrop` hears), with where the pointer is.
#[derive(Debug, Clone, PartialEq)]
pub struct FileDrop {
    /// The files, in the order the source listed them.
    pub paths: Vec<PathBuf>,
    /// Where the pointer is, or where they were let go (the window's logical pixels).
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
