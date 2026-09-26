//! An operation as the service reports it, and the token that lets a pending loop run
//! (design/26-DETAILS.md R4): a loop needs a [`PendingToken`], a token needs a start and a
//! deadline no later than `PendingCap`, and the loop holds its still frame at that deadline. There
//! is no way to write a pending loop without one.

use crate::appearance::MotionLevel;
use crate::tokens::DelayToken;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

/// How long a pending loop may play before it holds its still frame: at most `PendingCap`
/// (10 s). The operation itself may run longer; its own timeout is the service's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Deadline(Duration);

impl Deadline {
    /// `PendingCap`: the longest a loop plays.
    pub fn cap() -> Deadline {
        Deadline(DelayToken::PendingCap.delay(MotionLevel::Standard))
    }

    /// `length`, or the cap if it is longer: a deadline past `PendingCap` cannot be made.
    pub fn within(length: Duration) -> Deadline {
        Deadline(length.min(Deadline::cap().0))
    }

    /// The length.
    pub fn length(self) -> Duration {
        self.0
    }
}

/// A running operation's ticket: when it started and when its loop must stop. Made only by
/// [`PendingToken::start`], in the handler or service callback that started the operation;
/// each start is a new operation, so a new start restarts the grace and the cap.
///
/// ```
/// use ds::detail::{Deadline, Operation, PendingToken};
///
/// let joining = Operation::Running(PendingToken::start(Deadline::cap()));
/// # let _ = joining;
/// ```
///
/// ```compile_fail,E0451
/// // A token cannot be written by hand.
/// let token = ds::detail::PendingToken {
///     serial: 1,
///     started: std::time::Instant::now(),
///     deadline: ds::detail::Deadline::cap(),
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PendingToken {
    serial: u32,
    started: Instant,
    deadline: Deadline,
}

/// Tokens started so far in this process: each start is a distinct operation.
static STARTED: AtomicU32 = AtomicU32::new(0);

impl PendingToken {
    /// An operation starting now, whose loop stops at `deadline`. Call it where the operation is
    /// started (the handler that asked, the service event that reported it), not in render.
    pub fn start(deadline: Deadline) -> PendingToken {
        PendingToken {
            serial: STARTED.fetch_add(1, Ordering::Relaxed) + 1,
            started: crate::time::now(),
            deadline,
        }
    }

    /// When the operation started.
    pub fn started(self) -> Instant {
        self.started
    }

    /// When its loop holds still, from the start.
    pub fn deadline(self) -> Deadline {
        self.deadline
    }

    /// How long it has been running.
    pub fn elapsed(self) -> Duration {
        crate::time::since(self.started)
    }
}

/// An operation as the service reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Operation {
    /// Nothing is running.
    #[default]
    Idle,
    /// An operation is running under its token.
    Running(PendingToken),
}

#[cfg(test)]
mod tests {
    use super::{Deadline, PendingToken};
    use std::time::Duration;

    #[test]
    fn no_deadline_passes_the_cap() {
        let cap = Deadline::cap().length();
        assert_eq!(cap, Duration::from_secs(10));
        assert_eq!(Deadline::within(Duration::from_secs(60)).length(), cap);
        assert_eq!(
            Deadline::within(Duration::from_secs(2)).length(),
            Duration::from_secs(2)
        );
    }

    #[test]
    fn every_start_is_a_new_operation() {
        let a = PendingToken::start(Deadline::cap());
        let b = PendingToken::start(Deadline::cap());
        assert_ne!(a, b);
    }
}
