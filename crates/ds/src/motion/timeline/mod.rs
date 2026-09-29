//! Rust-driven animation as data. What the stylesheet cannot reach (an SVG arc, a count, a
//! spring's position, a pending loop's step) is a [`Timeline`]: a value that says how long it
//! runs and what a frame `elapsed` into it draws, as pure arithmetic a table pins. One driver,
//! [`playback::Playback`] (and [`use_timeline::use_timeline`], which follows a timeline its
//! caller recomputes), owns the
//! frame clock: it asks for a frame every `FRAME_TICK` while a timeline runs and never at rest
//! (design/26 R3).

pub(crate) mod count_up;
pub(crate) mod ease;
pub(crate) mod glide;
pub(crate) mod pending;
pub(crate) mod playback;
pub(crate) mod spring;
pub(crate) mod sweep;
pub(crate) mod use_timeline;

use std::time::Duration;

/// A motion as data.
pub(crate) trait Timeline: Clone + PartialEq + 'static {
    /// What one frame draws.
    type Frame: Clone + PartialEq + 'static;

    /// How long it runs before it stands still.
    fn total(&self) -> Duration;

    /// The frame `elapsed` into it; at and past [`Self::total`] the final one.
    fn at(&self, elapsed: Duration) -> Self::Frame;

    /// Whether it has finished by `elapsed`, so the driver stops asking for frames.
    fn settled(&self, elapsed: Duration) -> bool {
        elapsed >= self.total()
    }
}
