//! An operation as the service reports it, and the token that lets a pending loop run
//! (design/26-DETAILS.md R4): a loop needs a [`PendingToken`], and a token needs a start. There
//! is no way to write a pending loop without one.

use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

/// A running operation's ticket: when it started. Made only by [`PendingToken::start`], in the
/// handler or service callback that started the operation; each start is a new operation, so a
/// new start restarts the loop.
///
/// ```
/// use ds::detail::{Operation, PendingToken};
///
/// let joining = Operation::Running(PendingToken::start());
/// # let _ = joining;
/// ```
///
/// ```compile_fail,E0451
/// // A token cannot be written by hand.
/// let token = ds::detail::PendingToken {
///     serial: 1,
///     started: std::time::Instant::now(),
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PendingToken {
    serial: u32,
    started: Instant,
}

/// Tokens started so far in this process: each start is a distinct operation.
static STARTED: AtomicU32 = AtomicU32::new(0);

impl PendingToken {
    /// An operation starting now. Call it where the operation is started (the handler that
    /// asked, the service event that reported it), not in render.
    pub fn start() -> PendingToken {
        PendingToken {
            serial: STARTED.fetch_add(1, Ordering::Relaxed) + 1,
            started: ds_core::time::clock::now(),
        }
    }

    /// When the operation started.
    pub fn started(self) -> Instant {
        self.started
    }

    /// How long it has been running.
    pub fn elapsed(self) -> Duration {
        ds_core::time::clock::since(self.started)
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
    use super::PendingToken;

    #[test]
    fn every_start_is_a_new_operation() {
        let a = PendingToken::start();
        let b = PendingToken::start();
        assert_ne!(a, b);
    }
}
