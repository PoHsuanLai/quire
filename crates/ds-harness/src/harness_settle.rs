//! `Driver::advance` lets real wall-clock time pass (`harness` module doc): a test that asks
//! for exactly the time a settle takes can see it land early or late on a loaded machine. A test
//! that instead polls for the state it wants, up to a generous bound, and checks *when* that
//! landed relative to other events, survives load. [`settle_until`] is that poll, shared so every
//! test times a settle the same way instead of growing its own copy (as `cc_pane_switcher.rs` and
//! `coherence.rs` each once did).

use crate::driver::Driver;
use crate::harness::Harness;
use ds_core::time::clock::VirtualClock;
use std::time::{Duration, Instant};

/// How long a settled document must stay quiet for [`assert_settles_to_zero_frames`]: longer
/// than a pending step (`--t-pending-step`, 360 ms at Calm) and a frame, so a timer still stepping
/// is seen.
pub const QUIET: Duration = Duration::from_millis(500);

/// The most a settle may be stretched by a loaded machine before a test gives up: generous
/// against contention, short enough that a genuinely broken settle still fails promptly. Only
/// bounds the [`Clock::Wall`](crate::Clock::Wall) check; see [`VIRTUAL_DRAIN_BOUND`] for
/// [`Clock::Virtual`](crate::Clock::Virtual).
pub const SETTLE_BOUND: Duration = Duration::from_secs(3);

/// How far [`assert_settles_to_zero_frames`] may drain the virtual clock's pending sleeps,
/// earliest due first, before it gives up: generous against a long chain of holds, short enough
/// that a component that keeps rescheduling itself (a retrigger loop) still fails promptly rather
/// than hanging the test.
pub const VIRTUAL_DRAIN_BOUND: Duration = Duration::from_secs(30);

/// The step [`settle_until`] takes.
const STEP: Duration = Duration::from_millis(10);

/// How far [`settle_until`] advances next: [`STEP`], but on the virtual clock no further than the
/// next pending sleep's due instant, so a state a timer ends is seen at the very instant the
/// timer fires and not at the next step boundary after it.
fn step(harness: &Harness) -> Duration {
    harness
        .virtual_clock()
        .and_then(|clock| {
            clock
                .next_due()
                .map(|due| due.saturating_sub(clock.elapsed()))
        })
        .filter(|until| !until.is_zero())
        .map_or(STEP, |until| until.min(STEP))
}

/// Advance `harness` in 10 ms steps until `done` holds, for at most [`SETTLE_BOUND`] on the
/// harness's clock (wall or virtual, `Harness::now`). Returns the instant `done` first held, read
/// on that clock so a caller can compare it against other `Instant`s it took from `Harness::now` (e.g. "settled at least one full slide after it was
/// asked for"). Panics with the document's HTML — the last state `done` saw — if `done` never
/// holds within the bound, so a timeout is debuggable from the failure message alone.
///
/// **On [`Clock::Virtual`](crate::Clock::Virtual) the instant is exact when a timer ends the
/// state**: a step never runs past a pending sleep's due instant, so `done` is looked at the
/// moment the timer fired, and a test asserts the delay it measures with `assert_eq!`, not a
/// wall-clock hedge (`>=`) that a delay growing from 500 ms to 900 ms would still pass. A state
/// that only CSS time ends (an animation sampled between timers) is seen at the next 10 ms step.
pub fn settle_until(harness: &mut Harness, done: impl Fn(&Harness) -> bool) -> Instant {
    let started = harness.now();
    while harness.now().saturating_duration_since(started) < SETTLE_BOUND {
        if done(harness) {
            return harness.now();
        }
        let by = step(harness);
        harness.advance(by);
    }
    if done(harness) {
        return harness.now();
    }
    panic!(
        "settle_until: no state held within {SETTLE_BOUND:?}:\n{}",
        harness.html()
    );
}

/// Assert the idle-frame rule (design/26-DETAILS.md R3) on whatever `harness` shows now: the
/// document reaches a state where no CSS animation or transition runs (`is_animating() ==
/// false`) and no Rust timer wakes it for a whole [`QUIET`] window, and it stays that way. Every
/// component with a `Detailed` state calls it at the end of each moment's test. Panics with the
/// document's HTML if it never goes quiet.
///
/// **[`Clock::Virtual`](crate::Clock::Virtual) is exact.** `ds_core::time::clock::VirtualClock` knows the due
/// instant of every sleep still waiting (`next_due`), so this drains them: it advances to each
/// pending due instant in turn, earliest first, until none remain (bounded by
/// [`VIRTUAL_DRAIN_BOUND`] of virtual time — a component that keeps rescheduling itself fails
/// with every still-pending due time rather than hanging), then asserts a [`QUIET`] window with
/// no wake and no animation right after. "At rest" here means exactly design/26 R3: no CSS
/// animation, no pending `ds` timer, and nothing woke the document — not "quiet for one window",
/// which a later timer could still slip past.
///
/// **[`Clock::Wall`](crate::Clock::Wall) is a poll, not a drain.** A real harness cannot see what
/// is pending, only what just happened, so this instead advances in [`QUIET`] windows for up to
/// [`SETTLE_BOUND`] and returns as soon as *one* window shows no wake and no animation. That
/// window can pass quiet while a later timer is still asleep (e.g. a 900 ms hold started at the
/// top of a 500 ms window: the first window sees no wake yet and the check returns, even though
/// the hold is still pending) — found in the check mark's `SettleHold`.
/// Use `Clock::Virtual` (CONSUMING Rule 4) whenever the check must be strict about "nothing is
/// pending", not just "nothing fired in the window that happened to run".
pub fn assert_settles_to_zero_frames(harness: &mut Harness) {
    match harness.virtual_clock() {
        Some(clock) => assert_settles_virtual(harness, &clock),
        None => assert_settles_wall(harness),
    }
}

fn assert_settles_wall(harness: &mut Harness) {
    let started = harness.now();
    while harness.now().saturating_duration_since(started) < SETTLE_BOUND {
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

/// Drain every sleep pending on `clock`, advancing straight to each one's due instant (earliest
/// first, so an earlier timer's renders run before a later one is even looked for), then assert a
/// [`QUIET`] window with no wake and no animation right after the last one drains.
fn assert_settles_virtual(harness: &mut Harness, clock: &VirtualClock) {
    let started = clock.elapsed();
    while let Some(due) = clock.next_due() {
        if due.saturating_sub(started) > VIRTUAL_DRAIN_BOUND {
            panic!(
                "assert_settles_to_zero_frames: {} timer(s) still pending after \
                 {VIRTUAL_DRAIN_BOUND:?} of virtual time (due at {:?}, started at {started:?}) \
                 — a component may be rescheduling itself forever:\n{}",
                clock.waiting(),
                clock.due_times(),
                harness.html()
            );
        }
        harness.advance(due.saturating_sub(clock.elapsed()));
    }
    let wakes = harness.wakes();
    harness.advance(QUIET);
    if harness.wakes() != wakes || harness.is_animating() {
        panic!(
            "assert_settles_to_zero_frames: a timer or animation started during the {QUIET:?} \
             quiet window right after every pending sleep drained (animating: {}):\n{}",
            harness.is_animating(),
            harness.html()
        );
    }
}
