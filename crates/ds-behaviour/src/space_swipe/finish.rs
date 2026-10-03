//! The finish: an ease-out-cubic slide to a Space, sampled by the caller from the state and
//! read back here when a swipe or a `Go` catches it in flight.

use std::time::Duration;

use ds_core::time::stamp::Stamp;

use super::model::{PageMilli, PagesPerSecMilli, Reduced, Swipe, SwipeOut, SwipeParams};
use super::numbers::{div_nearest, finish_duration, saturate};
use crate::span;

/// What a step wants done, beside the state it leaves.
type Step = (Swipe, Vec<SwipeOut>);

/// A slide in flight: where it started, how fast, when, and the Space it ends on.
#[derive(Debug, Clone, Copy)]
pub(super) struct Slide {
    pub(super) to: u32,
    pub(super) p0: PageMilli,
    pub(super) v0: PagesPerSecMilli,
    pub(super) since: Stamp,
}

impl Slide {
    /// The distance still to cover from the start, in thousandths of a page.
    fn distance(&self) -> i64 {
        i64::from(PageMilli::of_space(self.to).0) - i64::from(self.p0.0)
    }

    /// How long the whole slide takes.
    fn duration(&self) -> Duration {
        finish_duration(PageMilli(saturate(self.distance())), self.v0)
    }

    /// When the slide reaches its Space.
    pub(super) fn ends(&self) -> Stamp {
        span::after(self.since, self.duration())
    }

    /// The slide's length and the time left in it at `now`, both in milliseconds.
    fn remaining(&self, now: Stamp) -> (i64, i64) {
        let total = i64::try_from(span::millis(self.duration())).unwrap_or(i64::MAX);
        let spent = i64::try_from(now.since(self.since))
            .unwrap_or(i64::MAX)
            .min(total);
        (total, total - spent)
    }

    /// Where the row is at `now`: `p0 + Δ (1 − (1 − u)³)`.
    pub(super) fn position(&self, now: Stamp) -> PageMilli {
        let (total, left) = self.remaining(now);
        let covered = total.pow(3) - left.pow(3);
        let travelled = div_nearest(self.distance() * covered, total.pow(3));
        PageMilli(saturate(i64::from(self.p0.0) + travelled))
    }

    /// How fast the row is going at `now`: `3 Δ (1 − u)² / T`, per second.
    pub(super) fn speed(&self, now: Stamp) -> PagesPerSecMilli {
        let (total, left) = self.remaining(now);
        let per_ms = self.distance() * 3 * left.pow(2);
        PagesPerSecMilli(saturate(div_nearest(per_ms * 1000, total.pow(3))))
    }
}

/// Starts the slide from Space `from` to Space `to` at `now`, from `p0` at speed `v0`. Under
/// reduced motion the row is at `to` already.
pub(super) fn begin(
    from: u32,
    to: u32,
    p0: PageMilli,
    v0: PagesPerSecMilli,
    now: Stamp,
    params: &SwipeParams,
) -> Step {
    match params.reduced {
        Reduced::Yes => (
            Swipe::Idle { at: to },
            vec![SwipeOut::Committed { to }, SwipeOut::Settled { at: to }],
        ),
        Reduced::No => (
            Swipe::Finishing {
                from,
                to,
                p0,
                v0,
                since: now,
            },
            vec![SwipeOut::Committed { to }],
        ),
    }
}
