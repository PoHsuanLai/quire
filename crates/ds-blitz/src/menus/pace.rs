//! How fast one sender may send `EventGroup`.

use super::permit::SenderName;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// The most events one `EventGroup` may carry. A person presses one item at a time; an agent
/// batching a few is fine, and a call of thousands only keeps the bus thread busy.
pub(super) const EVENTS_PER_CALL_MAX: usize = 64;
/// The most `EventGroup` calls one sender may make in a second: a peer past this is told to wait,
/// so a loop of calls cannot hold the thread that also answers the shell's layout reads.
pub(super) const CALLS_PER_SECOND_MAX: usize = 8;
/// The most senders remembered at once. The permit has already let each of them in, so this is a
/// few programs; a sender beyond it (after the idle are dropped) is told to wait.
const SENDERS_MAX: usize = 64;
/// The span the call count is taken over.
const WINDOW: Duration = Duration::from_secs(1);

/// Whether a call may go ahead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Pass {
    Go,
    Wait,
}

/// The recent `EventGroup` calls of each sender.
#[derive(Debug, Default)]
pub(super) struct Pace {
    calls: HashMap<SenderName, VecDeque<Instant>>,
}

impl Pace {
    /// Counts a call by `sender` at `now`, and says whether it is within the rate.
    pub(super) fn admit(&mut self, sender: &SenderName, now: Instant) -> Pass {
        if self.is_full_for(sender) {
            self.forget_idle(now);
        }
        if self.is_full_for(sender) {
            return Pass::Wait;
        }
        let recent = self.calls.entry(sender.clone()).or_default();
        while recent
            .front()
            .is_some_and(|at| now.duration_since(*at) >= WINDOW)
        {
            recent.pop_front();
        }
        if recent.len() >= CALLS_PER_SECOND_MAX {
            return Pass::Wait;
        }
        recent.push_back(now);
        Pass::Go
    }

    fn is_full_for(&self, sender: &SenderName) -> bool {
        self.calls.len() >= SENDERS_MAX && !self.calls.contains_key(sender)
    }

    fn forget_idle(&mut self, now: Instant) {
        self.calls.retain(|_, recent| {
            recent
                .back()
                .is_some_and(|at| now.duration_since(*at) < WINDOW)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sender(n: usize) -> SenderName {
        SenderName(format!(":1.{n}"))
    }

    #[test]
    fn a_sender_gets_its_calls_per_second_and_then_waits_until_the_window_passes() {
        let (mut pace, start) = (Pace::default(), Instant::now());
        for _ in 0..CALLS_PER_SECOND_MAX {
            assert_eq!(pace.admit(&sender(1), start), Pass::Go);
        }
        assert_eq!(pace.admit(&sender(1), start), Pass::Wait);
        assert_eq!(
            pace.admit(&sender(2), start),
            Pass::Go,
            "another sender has its own count"
        );
        assert_eq!(pace.admit(&sender(1), start + WINDOW), Pass::Go);
    }

    #[test]
    fn a_crowd_of_senders_is_told_to_wait_until_some_go_idle() {
        let (mut pace, start) = (Pace::default(), Instant::now());
        for n in 0..SENDERS_MAX {
            assert_eq!(pace.admit(&sender(n), start), Pass::Go);
        }
        assert_eq!(pace.admit(&sender(SENDERS_MAX), start), Pass::Wait);
        assert_eq!(pace.admit(&sender(0), start), Pass::Go, "a known sender");
        assert_eq!(pace.admit(&sender(SENDERS_MAX), start + WINDOW), Pass::Go);
    }
}
