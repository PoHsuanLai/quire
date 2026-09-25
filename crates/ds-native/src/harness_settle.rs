//! `Harness::advance` lets real wall-clock time pass (`harness` module doc): a test that asks
//! for exactly the time a settle takes can see it land early or late on a loaded machine. A test
//! that instead polls for the state it wants, up to a generous bound, and checks *when* that
//! landed relative to other events, survives load. [`settle_until`] is that poll, shared so every
//! test times a settle the same way instead of growing its own copy (as `cc_pane_switcher.rs` and
//! `coherence.rs` each once did).

use crate::harness::Harness;
use std::time::{Duration, Instant};

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
