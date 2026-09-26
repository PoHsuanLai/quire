//! A virtual timeline: an origin, how far it has been advanced, and the sleeps waiting on it.
//! Nothing here reads the wall clock after the origin; time moves only when
//! [`Timeline::advance_to`] is called.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};

/// One sleep's place in the queue: when it is due, then the order it was started in, so two
/// sleeps due at the same instant fire in the order they began.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct TimerKey {
    due: Duration,
    issued: u64,
}

/// The shared state of one virtual clock.
#[derive(Debug)]
pub(super) struct Timeline {
    origin: Instant,
    elapsed: Cell<Duration>,
    issued: Cell<u64>,
    /// Every sleep not yet finished, with the waker of the task awaiting it once it was polled.
    waiting: RefCell<BTreeMap<TimerKey, Option<Waker>>>,
}

impl Timeline {
    pub(super) fn new(origin: Instant) -> Self {
        Timeline {
            origin,
            elapsed: Cell::new(Duration::ZERO),
            issued: Cell::new(0),
            waiting: RefCell::new(BTreeMap::new()),
        }
    }

    pub(super) fn elapsed(&self) -> Duration {
        self.elapsed.get()
    }

    pub(super) fn now(&self) -> Instant {
        self.origin + self.elapsed.get()
    }

    /// The earliest instant a sleep is still waiting for, if any: the next step an advance
    /// takes. Sleeps already due (woken, not yet polled) are not counted.
    pub(super) fn next_due(&self) -> Option<Duration> {
        let elapsed = self.elapsed.get();
        self.waiting
            .borrow()
            .keys()
            .map(|key| key.due)
            .find(|due| *due > elapsed)
    }

    /// How many sleeps are waiting.
    pub(super) fn waiting(&self) -> usize {
        self.waiting.borrow().len()
    }

    /// Move to `at` (never backwards) and wake every sleep due by then, earliest first.
    pub(super) fn advance_to(&self, at: Duration) {
        let at = at.max(self.elapsed.get());
        self.elapsed.set(at);
        let due: Vec<Waker> = self
            .waiting
            .borrow_mut()
            .iter_mut()
            .take_while(|(key, _)| key.due <= at)
            .filter_map(|(_, waker)| waker.take())
            .collect();
        due.into_iter().for_each(Waker::wake);
    }

    fn enqueue(&self, length: Duration) -> TimerKey {
        let issued = self.issued.get();
        self.issued.set(issued + 1);
        let key = TimerKey {
            due: self.elapsed.get().saturating_add(length),
            issued,
        };
        self.waiting.borrow_mut().insert(key, None);
        key
    }

    fn poll_key(&self, key: TimerKey, waker: &Waker) -> Poll<()> {
        let mut waiting = self.waiting.borrow_mut();
        if key.due <= self.elapsed.get() {
            waiting.remove(&key);
            return Poll::Ready(());
        }
        waiting.insert(key, Some(waker.clone()));
        Poll::Pending
    }

    fn forget(&self, key: TimerKey) {
        self.waiting.borrow_mut().remove(&key);
    }
}

/// A sleep on a virtual timeline: ready once the timeline reaches its due instant. Dropping it
/// unfinished (a cancelled task) takes it out of the queue.
#[derive(Debug)]
pub(super) struct VirtualSleep {
    timeline: Rc<Timeline>,
    key: TimerKey,
}

impl VirtualSleep {
    pub(super) fn new(timeline: Rc<Timeline>, length: Duration) -> Self {
        let key = timeline.enqueue(length);
        VirtualSleep { timeline, key }
    }
}

impl Future for VirtualSleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        self.timeline.poll_key(self.key, cx.waker())
    }
}

impl Drop for VirtualSleep {
    fn drop(&mut self) {
        self.timeline.forget(self.key);
    }
}
