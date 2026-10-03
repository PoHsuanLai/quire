//! The swipe's state, units, inputs, outputs and settings.

use ds_core::machine::Elapsed;
use ds_core::time::stamp::Stamp;

/// A position on the row of Spaces, or a distance along it, in thousandths of a page; it grows
/// toward later Spaces, and Space `k` sits at `k × 1000`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct PageMilli(pub i32);

impl PageMilli {
    /// Where Space `k` sits on the row: `k × 1000`.
    pub fn of_space(k: u32) -> PageMilli {
        PageMilli(i32::try_from(k).unwrap_or(i32::MAX).saturating_mul(1000))
    }
}

/// A speed along the row of Spaces in thousandths of a page per second, positive toward later
/// Spaces (1.5 pages/s is `1500`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct PagesPerSecMilli(pub i32);

/// The recent `(time, raw position)` pairs a lift's velocity is fitted to; only the last 80 ms
/// matter.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Samples(pub(crate) Vec<(Stamp, PageMilli)>);

/// The swipe's state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Swipe {
    /// At rest on Space `at` (its index on the row).
    Idle {
        /// The shown Space.
        at: u32,
    },
    /// The fingers are down and the row follows them.
    Tracking {
        /// The Space the gesture started on; it moves at most one Space from here.
        from: u32,
        /// The device, which sets how far a page is.
        source: SwipeSource,
        /// Where the fingers have put the row, before the rubber band.
        p: PageMilli,
        /// Recent positions, for the velocity at lift.
        samples: Samples,
    },
    /// Sliding to `to` along an ease-out-cubic from `p0` at `since`, over
    /// [`finish_duration`](super::finish_duration)`(to × 1000 − p0, v0)`.
    Finishing {
        /// The Space the slide or gesture started on.
        from: u32,
        /// The Space it ends on.
        to: u32,
        /// Where the slide started (rubber band applied).
        p0: PageMilli,
        /// The speed it started with.
        v0: PagesPerSecMilli,
        /// When it started.
        since: Stamp,
    },
}

impl Default for Swipe {
    fn default() -> Swipe {
        Swipe::Idle { at: 0 }
    }
}

/// The device a swipe comes from, which sets the distance of one page: 1000 raw x units on the
/// Magic Mouse (palmrest's units), 400 px of libinput swipe delta on a touchpad.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SwipeSource {
    /// Two fingers locked horizontal on a Magic Mouse; one page is 1000 raw units.
    MagicMouse,
    /// Three or four fingers horizontal on a touchpad; one page is 400 px.
    Touchpad,
}

impl SwipeSource {
    /// How many of the device's units make one page.
    pub fn units_per_page(self) -> i32 {
        match self {
            SwipeSource::MagicMouse => 1000,
            SwipeSource::Touchpad => 400,
        }
    }
}

/// What moves the swipe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwipeIn {
    /// Fingers came down and locked into a horizontal swipe.
    Began {
        /// The device.
        source: SwipeSource,
    },
    /// The fingers moved `dx` in the device's units, positive to the right; the row follows the
    /// fingers, so a move to the left brings the next Space in.
    Changed {
        /// The horizontal move since the last change.
        dx: i32,
    },
    /// The fingers lifted.
    Ended,
    /// The gesture was abandoned (a click, a palm): back to where it started.
    Cancelled,
    /// Show Space `to` (a key, an intent, a click in the overview).
    Go {
        /// The Space's index on the row.
        to: u32,
    },
    /// The finish's end, asked for by the machine's wake, came due.
    Elapsed,
}

impl From<Elapsed> for SwipeIn {
    fn from(_: Elapsed) -> SwipeIn {
        SwipeIn::Elapsed
    }
}

/// What the swipe wants done.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwipeOut {
    /// Draw the row at `p` (rubber band applied).
    Offset {
        /// The row's position.
        p: PageMilli,
    },
    /// The target Space is decided (it may be the Space the gesture started on).
    Committed {
        /// The Space the row is going to.
        to: u32,
    },
    /// The row came to rest on Space `at`.
    Settled {
        /// The shown Space.
        at: u32,
    },
}

/// The swipe's settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwipeParams {
    /// How many Spaces the row holds; the last is `spaces − 1`.
    pub spaces: u32,
    /// Reduced motion: no live slide and no finish; a commit settles at once and the caller
    /// cross-fades (design/27 §3.1).
    pub reduced: Reduced,
}

/// Whether motion is reduced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Reduced {
    /// Full motion.
    No,
    /// Reduced motion.
    Yes,
}
