//! Waiting, the one way the design system spends time: a `futures-timer` sleep, never an ad-hoc
//! thread sleep (ORCHESTRATION coherence rule 4).
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

use std::time::Duration;

/// Added to every [`crate::settle`]: two frames at 60 Hz, so a timer never ends before the
/// last frame of the animation it waits for is painted.
pub const FRAME_SLACK: Duration = Duration::from_millis(34);

/// Wait `duration`. Start it from an event handler, not from render.
pub async fn sleep(duration: Duration) {
    todo!()
}
