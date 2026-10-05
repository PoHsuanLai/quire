//! Which clock this thread reads: the wall clock unless a [`VirtualClock`] is installed.

use super::virtual_queue::{VirtualQueue, VirtualSleep};
use std::cell::{Cell, RefCell};
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

/// One install on this thread: which guard made it, and the clock it chose (`None` is the
/// wall clock, put on the stack explicitly so it shadows a virtual clock installed below it).
#[derive(Debug)]
struct Install {
    id: u64,
    queue: Option<Rc<VirtualQueue>>,
}

/// Every clock installed on this thread, oldest first, and the id the next one gets. The last
/// entry is the clock in force; an empty stack is the wall clock. A guard removes its own entry
/// by id, so guards may drop in any order and the clock in force is always the newest one still
/// alive.
#[derive(Debug)]
struct Installs {
    next_id: u64,
    stack: Vec<Install>,
}

thread_local! {
    static INSTALLED: RefCell<Installs> = const {
        RefCell::new(Installs { next_id: 0, stack: Vec::new() })
    };
}

thread_local! {
    /// How many wall-clock sleeps have finished on this thread.
    static WALL_FINISHED: Cell<u64> = const { Cell::new(0) };
}

/// How many wall-clock sleeps have finished on this thread, counted when the sleep's owner
/// polls it and finds it done. A test host that bounds how many rounds of work a document may
/// take reads it to tell a round that real time made (a timer ran out) from one the document
/// made itself; a virtual sleep never counts, since virtual time only moves when the host says.
pub fn wall_sleeps_finished() -> u64 {
    WALL_FINISHED.with(Cell::get)
}

fn installed() -> Option<Rc<VirtualQueue>> {
    INSTALLED.with(|slot| slot.borrow().stack.last().and_then(|top| top.queue.clone()))
}

fn push(queue: Option<Rc<VirtualQueue>>) -> ClockGuard {
    INSTALLED.with(|slot| {
        let mut installs = slot.borrow_mut();
        let id = installs.next_id;
        installs.next_id += 1;
        installs.stack.push(Install { id, queue });
        ClockGuard {
            id,
            thread: PhantomData,
        }
    })
}

/// Make the wall clock the one [`now`] and [`sleep`] read on this thread until the guard drops,
/// even while a [`VirtualClock`] installed earlier is still alive: a wall-clock test host built
/// beside a virtual one must not read the virtual one's time.
pub fn install_wall() -> ClockGuard {
    push(None)
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
            Wait::Wall(delay) => {
                let polled = Pin::new(delay).poll(cx);
                if polled.is_ready() {
                    WALL_FINISHED.with(|done| done.set(done.get() + 1));
                }
                polled
            }
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

    /// Make this the clock [`now`] and [`sleep`] read on this thread until the guard drops.
    /// Guards may drop in any order: the clock in force is always the newest install still
    /// alive, and once none is, the wall clock.
    pub fn install(&self) -> ClockGuard {
        push(Some(Rc::clone(&self.queue)))
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
    /// settle check gives up on this clock (`ds_blitz::assert_settles_to_zero_frames`).
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

/// Keeps a clock installed on this thread; dropping it removes that install (and only that
/// one, whatever order guards drop in).
#[derive(Debug)]
#[must_use = "the clock is uninstalled as soon as the guard drops"]
pub struct ClockGuard {
    id: u64,
    /// The installs live in a thread-local, so the guard stays on its thread.
    thread: PhantomData<*const ()>,
}

impl Drop for ClockGuard {
    fn drop(&mut self) {
        let id = self.id;
        // `try_with`: a guard dropped during thread teardown finds the stack already gone.
        let _ =
            INSTALLED.try_with(|slot| slot.borrow_mut().stack.retain(|install| install.id != id));
    }
}
