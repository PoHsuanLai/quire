//! `Harness::advance` lets real wall-clock time pass (`harness` module doc): a test that asks
//! for exactly the time a settle takes can see it land early or late on a loaded machine. A test
//! that instead polls for the state it wants, up to a generous bound, and checks *when* that
//! landed relative to other events, survives load. [`settle_until`] is that poll, shared so every
//! test times a settle the same way instead of growing its own copy (as `cc_pane_switcher.rs` and
//! `coherence.rs` each once did).

use crate::harness::Harness;
use std::time::{Duration, Instant};

/// How long a settled document must stay quiet for [`assert_settles_to_zero_frames`]: longer
/// than a pending step (`--t-pending-step`, 360 ms at Calm) and a frame, so a timer still stepping
/// is seen.
pub const QUIET: Duration = Duration::from_millis(500);

/// The most a settle may be stretched by a loaded machine before a test gives up: generous
/// against contention, short enough that a genuinely broken settle still fails promptly.
pub const SETTLE_BOUND: Duration = Duration::from_secs(3);

/// Advance `harness` in 10 ms steps until `done` holds, for at most [`SETTLE_BOUND`] of wall
/// clock. Returns the instant `done` first held, read on the wall clock so a caller can compare
/// it against other `Instant`s it took (e.g. "settled at least one full slide after it was
/// asked for"). Panics with the document's HTML — the last state `done` saw — if `done` never
/// holds within the bound, so a timeout is debuggable from the failure message alone.
pub fn settle_until(harness: &mut Harness, done: impl Fn(&Harness) -> bool) -> Instant {
    let started = Instant::now();
    while started.elapsed() < SETTLE_BOUND {
        if done(harness) {
            return Instant::now();
        }
        harness.advance(Duration::from_millis(10));
    }
    if done(harness) {
        return Instant::now();
    }
    panic!(
        "settle_until: no state held within {SETTLE_BOUND:?}:\n{}",
        harness.html()
    );
}

/// Assert the idle-frame rule (design/26-DETAILS.md R3) on whatever `harness` shows now: within
/// [`SETTLE_BOUND`] of wall clock the document reaches a state where no CSS animation or transition
/// runs (`is_animating() == false`) and no Rust timer wakes it for a whole [`QUIET`] window, and
/// it stays that way. Every component with a `Detailed` state calls it at the end of each
/// moment's test. Panics with the document's HTML if it never goes quiet.
pub fn assert_settles_to_zero_frames(harness: &mut Harness) {
    let started = Instant::now();
    while started.elapsed() < SETTLE_BOUND {
        let wakes = harness.wakes();
        harness.advance(QUIET);
        if harness.wakes() == wakes && !harness.is_animating() {
            return;
        }
    }
    panic!(
        "assert_settles_to_zero_frames: still asking for frames after {SETTLE_BOUND:?} \
         (animating: {}):\n{}",
        harness.is_animating(),
        harness.html()
    );
}
