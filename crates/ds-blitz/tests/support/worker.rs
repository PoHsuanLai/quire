//! Waiting on a real thread from a harness on the virtual clock.
//!
//! The virtual clock's `advance` takes no wall time, so a state that a worker thread ends (the
//! spell checker's reply, a PDF raster) cannot be waited out in virtual time: `settle_until`
//! would spend its whole bound before the thread ran. These helpers wait on the event instead,
//! giving the thread real time between looks, and bound the wait with a hang guard that no
//! passing run comes near.

#![allow(dead_code)] // Each test binary that includes this file uses part of it.

use ds_harness::{Driver, Harness};
use std::time::{Duration, Instant};

/// How long a test waits on a worker before it calls the worker hung. A passing run returns the
/// moment the worker has answered and never waits on this.
pub const HANG_GUARD: Duration = Duration::from_secs(60);

/// The real time between looks at a worker's progress.
const LOOK: Duration = Duration::from_millis(1);

/// The virtual time `settle_on_worker` lets pass per look, for timers the app runs meanwhile.
const STEP: Duration = Duration::from_millis(10);

/// Wait for `done`, moving virtual time on in [`STEP`]s between looks so the app's own timers
/// (a debounce) run while the worker works.
pub fn settle_on_worker(harness: &mut Harness, done: impl Fn(&Harness) -> bool) {
    wait(harness, STEP, done);
}

/// Wait for `done` without moving the clock: the instant it first held at is the instant the
/// test left the clock on, so a test can assert the exact virtual time the reply arrived at.
pub fn wait_for_reply(harness: &mut Harness, done: impl Fn(&Harness) -> bool) {
    wait(harness, Duration::ZERO, done);
}

/// Let the worker have `grace` of real time, then run what it woke. For a check that it must
/// not have answered: it can only let a wrong answer through, never fail a right one.
pub fn give_worker_time(harness: &mut Harness, grace: Duration) {
    std::thread::sleep(grace);
    harness.advance(Duration::ZERO);
}

fn wait(harness: &mut Harness, step: Duration, done: impl Fn(&Harness) -> bool) {
    let began = Instant::now();
    while !done(harness) {
        assert!(
            began.elapsed() < HANG_GUARD,
            "the worker never answered:\n{}",
            harness.html()
        );
        harness.advance(step);
        std::thread::sleep(LOOK);
    }
}
