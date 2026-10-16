//! Availability changes: a log every subscriber reads at its own pace, woken by a `Waker`. No
//! runtime behind it, so any executor drives it.

use crate::capability::Capability;
use crate::probe::Presence;
use std::collections::HashMap;
use std::future::poll_fn;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::task::{Poll, Waker};

/// A tool became available, or went away.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Availability {
    /// The capability whose tool changed.
    pub capability: Capability,
    /// How it is now.
    pub presence: Presence,
}

#[derive(Debug, Default)]
struct State {
    known: HashMap<Capability, Presence>,
    log: Vec<Availability>,
    wakers: Vec<Waker>,
}

/// The shared log. Cheap to clone.
#[derive(Debug, Clone, Default)]
pub(crate) struct Log(Arc<Mutex<State>>);

impl Log {
    fn state(&self) -> MutexGuard<'_, State> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Record `presence`; announce it when it differs from the last one recorded (the first
    /// sighting of a capability is announced only when the tool is present).
    pub(crate) fn record(&self, capability: &Capability, presence: Presence) {
        let wakers = {
            let mut state = self.state();
            let before = state.known.insert(capability.clone(), presence);
            let changed = match before {
                Some(before) => before != presence,
                None => presence == Presence::Present,
            };
            if !changed {
                return;
            }
            state.log.push(Availability {
                capability: capability.clone(),
                presence,
            });
            std::mem::take(&mut state.wakers)
        };
        wakers.into_iter().for_each(Waker::wake);
    }

    pub(crate) fn subscribe(&self) -> Subscription {
        let seen = self.state().log.len();
        Subscription {
            log: self.clone(),
            seen,
        }
    }
}

/// A feed of availability changes from the moment it was made. Await [`Subscription::next`] in
/// whatever task suits; it never ends while the feed's [`crate::Helpers`] lives.
#[derive(Debug)]
pub struct Subscription {
    log: Log,
    seen: usize,
}

impl Subscription {
    /// The next change.
    pub async fn next(&mut self) -> Availability {
        poll_fn(|cx| {
            let mut state = self.log.state();
            match state.log.get(self.seen).cloned() {
                Some(change) => {
                    self.seen += 1;
                    Poll::Ready(change)
                }
                None => {
                    // A select loop polls again and again with the same waker: keep one.
                    if !state.wakers.iter().any(|w| w.will_wake(cx.waker())) {
                        state.wakers.push(cx.waker().clone());
                    }
                    Poll::Pending
                }
            }
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::Log;
    use std::future::Future;
    use std::pin::pin;
    use std::sync::Arc;
    use std::task::{Context, Wake, Waker};

    /// A waker with one address (`Waker::noop` may not compare equal to itself).
    struct Quiet;

    impl Wake for Quiet {
        fn wake(self: Arc<Self>) {}
    }

    #[test]
    fn polling_again_with_the_same_waker_keeps_one_waker() {
        let log = Log::default();
        let mut feed = log.subscribe();
        let mut next = pin!(feed.next());
        let waker = Waker::from(Arc::new(Quiet));
        let mut cx = Context::from_waker(&waker);
        for _ in 0..50 {
            assert!(next.as_mut().poll(&mut cx).is_pending());
        }
        assert_eq!(log.state().wakers.len(), 1);
    }
}
