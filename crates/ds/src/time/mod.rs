//! Time, the one way the design system reads it: [`now`] for "when is it", [`sleep`] for "wait
//! this long", never `Instant::now()` or an ad-hoc thread sleep (ORCHESTRATION coherence rule 4).
//! Spawning the task that waits is `crate::task`.
//!
//! Both read the clock installed on this thread: the wall clock by default, or a
//! [`VirtualClock`] a test harness installs so that every timer and every "now" in the design
//! system moves only when the test advances it (sill Q380). A loaded machine then changes how
//! long a test takes, never what it sees.

mod clock;
#[cfg(test)]
mod tests;
mod timeline;

pub use clock::{ClockGuard, VirtualClock, now, since, sleep};

use std::time::Duration;

/// Added to every [`crate::settle`]: two frames at 60 Hz, so a timer never ends before the
/// last frame of the animation it waits for is painted.
pub const FRAME_SLACK: Duration = Duration::from_millis(34);

/// One frame at 60 Hz, rounded down: how often a motion driven from Rust (a battery ring's
/// sweep, [`crate::motion::use_level_run`]) recomputes what it draws. Not a design duration: the
/// motion's length comes from its token; this is only the sampling rate.
pub const FRAME_TICK: Duration = Duration::from_millis(16);
