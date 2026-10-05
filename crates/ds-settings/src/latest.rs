//! The newest value one task publishes and another awaits, with no runtime behind it: a
//! `Mutex`, a version and the waiting task's `Waker`. Every consumer of a settings change is an
//! async task somewhere (a Dioxus future, a Tokio task), so the wake-up is all this needs.

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::task::{Context, Poll, Waker};

/// Whether one end of the channel is still there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Link {
    Open,
    Closed,
}

#[derive(Debug)]
struct Slot<T> {
    value: T,
    version: u64,
    sender: Link,
    receiver: Link,
    wakers: Vec<Waker>,
}

#[derive(Debug)]
struct Shared<T>(Mutex<Slot<T>>);

impl<T> Shared<T> {
    fn slot(&self) -> MutexGuard<'_, Slot<T>> {
        // A poisoned lock only means another thread panicked mid-publish; the slot is still a
        // whole value.
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// The publishing end.
#[derive(Debug)]
pub(crate) struct Sender<T>(Arc<Shared<T>>);

/// The awaiting end.
#[derive(Debug)]
pub(crate) struct Receiver<T> {
    shared: Arc<Shared<T>>,
    seen: u64,
}

/// A channel holding `initial`, which the receiver has already seen.
pub(crate) fn channel<T>(initial: T) -> (Sender<T>, Receiver<T>) {
    let shared = Arc::new(Shared(Mutex::new(Slot {
        value: initial,
        version: 0,
        sender: Link::Open,
        receiver: Link::Open,
        wakers: Vec::new(),
    })));
    (Sender(shared.clone()), Receiver { shared, seen: 0 })
}

/// The receiver is gone: nobody is left to publish for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ReceiverGone;

impl<T> Sender<T> {
    /// Publish `value`, waking the receiver.
    pub(crate) fn send(&self, value: T) -> Result<(), ReceiverGone> {
        let wakers = {
            let mut slot = self.0.slot();
            if slot.receiver == Link::Closed {
                return Err(ReceiverGone);
            }
            slot.value = value;
            slot.version += 1;
            std::mem::take(&mut slot.wakers)
        };
        wakers.into_iter().for_each(Waker::wake);
        Ok(())
    }

    /// The value the receiver would read now.
    pub(crate) fn latest(&self) -> T
    where
        T: Clone,
    {
        self.0.slot().value.clone()
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let wakers = {
            let mut slot = self.0.slot();
            slot.sender = Link::Closed;
            std::mem::take(&mut slot.wakers)
        };
        wakers.into_iter().for_each(Waker::wake);
    }
}

impl<T: Clone> Receiver<T> {
    /// The newest value, seen or not.
    pub(crate) fn latest(&self) -> T {
        self.shared.slot().value.clone()
    }

    /// Whether a value newer than the last one this receiver took has been published.
    pub(crate) fn has_changed(&self) -> bool {
        self.shared.slot().version > self.seen
    }

    /// Take the newest value as seen without waiting for it.
    pub(crate) fn catch_up(&mut self) {
        self.seen = self.shared.slot().version;
    }

    /// Wait for a value newer than the last one taken; `None` once the sender is gone and
    /// nothing newer is left.
    pub(crate) fn changed(&mut self) -> Changed<'_, T> {
        Changed { receiver: self }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.shared.slot().receiver = Link::Closed;
    }
}

/// The future [`Receiver::changed`] returns.
#[derive(Debug)]
pub(crate) struct Changed<'a, T> {
    receiver: &'a mut Receiver<T>,
}

impl<T: Clone> Future for Changed<'_, T> {
    type Output = Option<T>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<T>> {
        let receiver = &mut *self.receiver;
        let mut slot = receiver.shared.slot();
        if slot.version > receiver.seen {
            receiver.seen = slot.version;
            return Poll::Ready(Some(slot.value.clone()));
        }
        if slot.sender == Link::Closed {
            return Poll::Ready(None);
        }
        if !slot.wakers.iter().any(|waker| waker.will_wake(cx.waker())) {
            slot.wakers.push(cx.waker().clone());
        }
        Poll::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::{ReceiverGone, channel};

    #[tokio::test]
    async fn a_receiver_wakes_on_the_next_send_and_reads_the_newest() {
        let (tx, mut rx) = channel(0_u32);
        assert!(!rx.has_changed(), "the initial value is already seen");
        let waiting = tokio::spawn(async move { rx.changed().await });
        // One yield runs the spawned waiter up to its first await, so it is parked when the sends
        // come (a current-thread runtime): no wall-clock wait.
        tokio::task::yield_now().await;
        assert_eq!(tx.send(1), Ok(()));
        assert_eq!(tx.send(2), Ok(()));
        let got = waiting.await.unwrap_or_else(|e| panic!("{e}"));
        assert!(matches!(got, Some(1 | 2)), "{got:?}");
    }

    #[tokio::test]
    async fn two_sends_before_a_read_are_one_change_holding_the_newest() {
        let (tx, mut rx) = channel(0_u32);
        assert_eq!(tx.send(1), Ok(()));
        assert_eq!(tx.send(2), Ok(()));
        assert_eq!(rx.changed().await, Some(2));
        assert!(!rx.has_changed(), "nothing newer than the one taken");
    }

    #[tokio::test]
    async fn a_dropped_sender_ends_the_wait_after_the_last_value() {
        let (tx, mut rx) = channel(0_u32);
        assert_eq!(tx.send(7), Ok(()));
        drop(tx);
        assert_eq!(rx.changed().await, Some(7));
        assert_eq!(rx.changed().await, None);
    }

    #[test]
    fn a_dropped_receiver_refuses_the_next_send() {
        let (tx, rx) = channel(0_u32);
        drop(rx);
        assert_eq!(tx.send(1), Err(ReceiverGone));
    }
}
