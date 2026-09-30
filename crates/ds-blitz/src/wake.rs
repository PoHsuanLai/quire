//! A waker a headless document can sleep on: timers (`futures-timer`) and the net provider wake
//! it, and the harness waits on it instead of polling in a loop.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Condvar, Mutex, PoisonError};
use std::task::Wake;
use std::time::Duration;

/// Counts wake-ups and lets one thread wait for the next.
#[derive(Debug, Default)]
pub(crate) struct Wakeup {
    /// Every wake so far, from any source.
    generation: Mutex<u64>,
    changed: Condvar,
    /// Resources handed to the document so far.
    fetched: AtomicU64,
}

impl Wakeup {
    /// How many wakes have happened; pass it to [`Wakeup::wait_past`].
    pub(crate) fn generation(&self) -> u64 {
        *self
            .generation
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    /// Sleep until a wake after `seen`, or `limit` has passed.
    pub(crate) fn wait_past(&self, seen: u64, limit: Duration) {
        let guard = self
            .generation
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let _woken = self
            .changed
            .wait_timeout_while(guard, limit, |generation| *generation == seen)
            .unwrap_or_else(PoisonError::into_inner);
    }

    /// How many resources have landed so far.
    pub(crate) fn fetched(&self) -> u64 {
        self.fetched.load(Ordering::SeqCst)
    }

    /// A resource landed.
    pub(crate) fn note_fetch(&self) {
        self.fetched.fetch_add(1, Ordering::SeqCst);
        self.bump();
    }

    fn bump(&self) {
        let mut generation = self
            .generation
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        *generation += 1;
        self.changed.notify_all();
    }
}

impl Wake for Wakeup {
    fn wake(self: std::sync::Arc<Self>) {
        self.bump();
    }

    fn wake_by_ref(self: &std::sync::Arc<Self>) {
        self.bump();
    }
}

#[cfg(test)]
mod tests {
    use super::Wakeup;
    use std::sync::Arc;
    use std::task::Waker;
    use std::time::{Duration, Instant};

    #[test]
    fn a_wake_ends_the_wait_early() {
        let wakeup = Arc::new(Wakeup::default());
        let seen = wakeup.generation();
        let waker = Waker::from(Arc::clone(&wakeup));
        let started = Instant::now();
        let thread = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(20));
            waker.wake();
        });
        wakeup.wait_past(seen, Duration::from_secs(5));
        assert!(started.elapsed() < Duration::from_secs(2));
        assert_eq!(wakeup.generation(), seen + 1);
        thread.join().ok();
    }

    #[test]
    fn a_wait_without_a_wake_lasts_its_limit() {
        let wakeup = Wakeup::default();
        let started = Instant::now();
        wakeup.wait_past(wakeup.generation(), Duration::from_millis(30));
        assert!(started.elapsed() >= Duration::from_millis(30));
    }

    #[test]
    fn a_fetch_counts_and_wakes() {
        let wakeup = Wakeup::default();
        let (seen, fetched) = (wakeup.generation(), wakeup.fetched());
        wakeup.note_fetch();
        assert_eq!(wakeup.fetched(), fetched + 1);
        assert_eq!(wakeup.generation(), seen + 1);
    }
}
