//! Waiting, the one way the design system spends time: a `futures-timer` sleep, never an ad-hoc
//! thread sleep (ORCHESTRATION coherence rule 4). Spawning the task that waits is `crate::task`.

use std::time::Duration;

/// Added to every [`crate::settle`]: two frames at 60 Hz, so a timer never ends before the
/// last frame of the animation it waits for is painted.
pub const FRAME_SLACK: Duration = Duration::from_millis(34);

/// One frame at 60 Hz, rounded down: how often a motion driven from Rust (a battery ring's
/// sweep, [`crate::motion::use_level_run`]) recomputes what it draws. Not a design duration: the
/// motion's length comes from its token; this is only the sampling rate.
pub const FRAME_TICK: Duration = Duration::from_millis(16);

/// Wait `duration`. Start it from an event handler, not from render.
pub async fn sleep(duration: Duration) {
    futures_timer::Delay::new(duration).await;
}
