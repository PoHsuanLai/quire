//! Which clock this thread reads: the wall clock unless a [`VirtualClock`] is installed.

use super::virtual_queue::{VirtualQueue, VirtualSleep};
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

thread_local! {
    /// The virtual queue installed on this thread; `None` is the wall clock.
    static INSTALLED: RefCell<Option<Rc<VirtualQueue>>> = const { RefCell::new(None) };
}

fn installed() -> Option<Rc<VirtualQueue>> {
    INSTALLED.with(|slot| slot.borrow().clone())
}

/// Now, on this thread's clock: `Instant::now()` on the wall clock, or the virtual clock's
/// origin plus every advance so far.
pub fn now() -> Instant {
    installed().map_or_else(Instant::now, |queue| queue.now())
}

/// How long since `then`, on this thread's clock (zero if `then` is later): what
/// `then.elapsed()` would say, read from the clock [`now`] reads.
pub fn since(then: Instant) -> Duration {
    now().saturating_duration_since(then)
}

/// Wait `duration` on this thread's clock, measured from the call. Start it from an event
/// handler, not from render.
pub fn sleep(duration: Duration) -> impl Future<Output = ()> {
    match installed() {
        Some(queue) => Wait::Virtual(VirtualSleep::new(queue, duration)),
        None => Wait::Wall(futures_timer::Delay::new(duration)),
    }
}

/// A sleep on whichever clock was installed when it began.
#[derive(Debug)]
enum Wait {
    Wall(futures_timer::Delay),
    Virtual(VirtualSleep),
}

impl Future for Wait {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        match self.get_mut() {
            Wait::Wall(delay) => Pin::new(delay).poll(cx),
            Wait::Virtual(sleep) => Pin::new(sleep).poll(cx),
        }
    }
}

/// A clock that moves only when told to: a test harness installs one so that [`now`] and
/// [`sleep`] across the design system (motion timers, presence, hover intent, toast holds,
/// pending, detail tweens) follow the test's time instead of the machine's.
///
/// It starts at the wall-clock instant it was made and stands still there until
/// [`VirtualClock::advance_to`]. The clock and every sleep on it stay on the thread that made
/// them, which is the thread a Dioxus document polls its tasks on.
#[derive(Debug, Clone)]
pub struct VirtualClock {
    queue: Rc<VirtualQueue>,
}

impl VirtualClock {
    /// A clock standing at the wall clock's now.
    pub fn new() -> Self {
        VirtualClock {
            queue: Rc::new(VirtualQueue::new(Instant::now())),
        }
    }

    /// Make this the clock [`now`] and [`sleep`] read on this thread until the guard drops,
    /// which puts back whatever was installed before (so harnesses nest).
    pub fn install(&self) -> ClockGuard {
        let previous = INSTALLED.with(|slot| slot.replace(Some(Rc::clone(&self.queue))));
        ClockGuard { previous }
    }

    /// Now on this clock.
    pub fn now(&self) -> Instant {
        self.queue.now()
    }

    /// How far the clock has been advanced since it was made.
    pub fn elapsed(&self) -> Duration {
        self.queue.elapsed()
    }

    /// When the next sleep on this clock is due, as time since it was made; `None` when
    /// nothing is waiting.
    pub fn next_due(&self) -> Option<Duration> {
        self.queue.next_due()
    }

    /// How many sleeps on this clock have not finished.
    pub fn waiting(&self) -> usize {
        self.queue.waiting()
    }

    /// The due instant of every sleep still waiting, earliest first: for a diagnostic when a
    /// settle check gives up on this clock (`ds_native::assert_settles_to_zero_frames`).
    pub fn due_times(&self) -> Vec<Duration> {
        self.queue.due_times()
    }

    /// Move the clock to `at` after it was made (never backwards) and wake every sleep due by
    /// then, earliest first. The woken tasks run when their executor is next polled; a caller
    /// that wants each timer to see the state the previous one left steps through
    /// [`VirtualClock::next_due`] one instant at a time, polling in between.
    pub fn advance_to(&self, at: Duration) {
        self.queue.advance_to(at);
    }
}

impl Default for VirtualClock {
    fn default() -> Self {
        VirtualClock::new()
    }
}

/// Keeps a [`VirtualClock`] installed on this thread; dropping it restores the clock that was
/// installed before.
#[derive(Debug)]
#[must_use = "the clock is uninstalled as soon as the guard drops"]
pub struct ClockGuard {
    previous: Option<Rc<VirtualQueue>>,
}

impl Drop for ClockGuard {
    fn drop(&mut self) {
        let previous = self.previous.take();
        INSTALLED.with(|slot| slot.replace(previous));
    }
}
