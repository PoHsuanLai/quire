//! Which clock a [`Harness`](crate::Harness) runs on (sill Q380).
//!
//! On the [`Clock::Wall`] (the default) quire's timers are real `futures-timer` sleeps and
//! `Harness::advance` really lets the time pass; only CSS animation time is the harness's own.
//! Under load the two drift apart, so a timer can end an entrance before or after the frame a
//! test expects.
//!
//! On the [`Clock::Virtual`] the harness installs a [`ds::VirtualClock`] on its thread for as
//! long as it lives, so every `ds::time::now` and `ds::sleep` (motion timers, presence, hover
//! intent, toast holds, pending, detail tweens) reads the same clock the CSS resolves at, and
//! `advance` moves that one clock: it steps to each timer's due instant in order, fires what is
//! due, runs the renders that queued and resolves the document at that very instant. Nothing
//! waits on the wall clock, so what a test sees depends only on what it did.
//!
//! What the virtual clock does not reach: work off the harness's thread (a Tokio task such as
//! `ds_settings`' file watch, a D-Bus reply), which still runs on real time. `advance` on the
//! virtual clock never sleeps, so a test waiting for such work needs the wall clock. And Blitz's
//! own clock reads (rev e99fbdbd, `pub(crate)` fields a host cannot set): a press within 500 ms
//! of wall time after the last one at the same spot is a double click, and a scrollbar's fade,
//! both on the wall clock. On the virtual clock `advance` takes no wall time, so two clicks at
//! one spot are always a double click, however far apart the test advanced them.

use crate::harness::Harness;
use ds::{ClockGuard, VirtualClock};
use std::time::{Duration, Instant};

/// The clock a harness's timers run on; pass it to
/// [`HarnessConfig::with_clock`](crate::HarnessConfig::with_clock).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Clock {
    /// Real time: timers fire as the machine's clock passes, `advance` takes as long as it says.
    #[default]
    Wall,
    /// The harness's own time: `advance` moves one clock that drives both CSS animations and
    /// every ds timer, deterministically and at once.
    Virtual,
}

/// The clock a running harness holds.
#[derive(Debug)]
pub(crate) enum HarnessClock {
    Wall,
    /// The virtual clock, installed on the harness's thread until the guard drops.
    Virtual {
        clock: VirtualClock,
        _installed: ClockGuard,
    },
}

impl HarnessClock {
    /// Start `choice` on this thread. Call it before the document is built, so its first
    /// render already reads the harness's clock.
    pub(crate) fn start(choice: Clock) -> Self {
        match choice {
            Clock::Wall => HarnessClock::Wall,
            Clock::Virtual => {
                let clock = VirtualClock::new();
                let installed = clock.install();
                HarnessClock::Virtual {
                    clock,
                    _installed: installed,
                }
            }
        }
    }

    /// Which clock this is.
    pub(crate) fn choice(&self) -> Clock {
        match self {
            HarnessClock::Wall => Clock::Wall,
            HarnessClock::Virtual { .. } => Clock::Virtual,
        }
    }

    /// Now on this clock.
    pub(crate) fn now(&self) -> Instant {
        match self {
            HarnessClock::Wall => Instant::now(),
            HarnessClock::Virtual { clock, .. } => clock.now(),
        }
    }

    /// The virtual clock, if this is one.
    pub(crate) fn virtual_clock(&self) -> Option<VirtualClock> {
        match self {
            HarnessClock::Wall => None,
            HarnessClock::Virtual { clock, .. } => Some(clock.clone()),
        }
    }
}

/// Let `time` pass on `clock`: stop at each timer due before the end, then at the end, moving
/// the clock and resolving `harness` at every stop, so each timer sees the renders the one
/// before it caused and the CSS resolves at the instant the timer fired.
pub(crate) fn advance(harness: &mut Harness, clock: &VirtualClock, time: Duration) {
    let end = clock.elapsed().saturating_add(time);
    loop {
        let stop = next_stop(clock, end);
        let (Stop::Timer(at) | Stop::End(at)) = stop;
        clock.advance_to(at);
        harness.resolve_at(at);
        if stop == Stop::End(at) {
            return;
        }
    }
}

/// The instants one virtual `advance` stops at, from the clock's now to `time` later: each
/// timer's due instant in turn, found only after the previous one's renders ran (a timer can
/// start another), then the end.
fn next_stop(clock: &VirtualClock, end: Duration) -> Stop {
    match clock.next_due() {
        Some(due) if due < end => Stop::Timer(due),
        _ => Stop::End(end),
    }
}

/// Where a virtual advance stops next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stop {
    /// A timer is due here, before the end.
    Timer(Duration),
    /// The end of the advance (timers due exactly there fire too).
    End(Duration),
}
