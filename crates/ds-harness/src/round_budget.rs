//! How many rounds of work a document may take without input before the harness calls it a
//! render loop.
//!
//! A loop is rounds that make their own work: an effect that rewrites the signal it reads. On
//! the wall clock a round can also be made by real time moving: a ds timer ran out during the
//! previous round, and on a loaded machine one does every round (a 16 ms tick against rounds
//! that take longer). That is time passing, not a loop, so a round in which a wall-clock sleep
//! finished ([`wall_sleeps_finished`]) is not counted. A true loop finishes no sleeps, so it is
//! counted on either clock; the total cap still ends a loop that a steady timer hides. (Waker
//! wakes cannot tell the two apart: a timer that fires while a render runs queues its task
//! without waking anyone.)

use ds_core::time::clock::wall_sleeps_finished;

/// Rounds that made their own work, counted before the harness panics.
pub(crate) const MAX_ROUNDS: usize = 64;

/// Every round, counted or not, before the harness panics whatever woke them.
const MAX_TOTAL: usize = MAX_ROUNDS * 16;

/// What a round that produced more work leaves of the budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Spent {
    Within,
    Over,
}

/// The rounds one `flush` or one `frame` has spent.
#[derive(Debug)]
pub(crate) struct RoundBudget {
    sleeps_seen: u64,
    own: usize,
    total: usize,
}

impl RoundBudget {
    pub(crate) fn new() -> Self {
        RoundBudget {
            sleeps_seen: wall_sleeps_finished(),
            own: 0,
            total: 0,
        }
    }

    /// Record a round that produced more work.
    pub(crate) fn spend(&mut self) -> Spent {
        let sleeps = wall_sleeps_finished();
        let driven_by_time = sleeps != self.sleeps_seen;
        self.sleeps_seen = sleeps;
        self.total += 1;
        if !driven_by_time {
            self.own += 1;
        }
        if self.own >= MAX_ROUNDS || self.total >= MAX_TOTAL {
            Spent::Over
        } else {
            Spent::Within
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_ROUNDS, MAX_TOTAL, RoundBudget, Spent};
    use ds_core::time::clock::{install_wall, sleep};
    use std::future::Future;
    use std::pin::pin;
    use std::task::{Context, Waker};
    use std::time::Duration;

    /// A wall sleep that finishes now, counted on this thread.
    fn finish_a_sleep() {
        let _wall = install_wall();
        let mut cx = Context::from_waker(Waker::noop());
        let mut wait = pin!(sleep(Duration::ZERO));
        while wait.as_mut().poll(&mut cx).is_pending() {
            std::thread::yield_now();
        }
    }

    #[test]
    fn rounds_of_the_documents_own_work_run_out() {
        let mut budget = RoundBudget::new();
        let spent: Vec<Spent> = (0..MAX_ROUNDS).map(|_| budget.spend()).collect();
        assert!(spent[..MAX_ROUNDS - 1].iter().all(|s| *s == Spent::Within));
        assert_eq!(spent[MAX_ROUNDS - 1], Spent::Over);
    }

    #[test]
    fn a_round_a_timer_ran_out_in_is_not_counted() {
        let mut budget = RoundBudget::new();
        for _ in 0..MAX_ROUNDS * 4 {
            finish_a_sleep();
            assert_eq!(budget.spend(), Spent::Within);
        }
    }

    #[test]
    fn a_steady_timer_does_not_hide_a_loop_for_ever() {
        let mut budget = RoundBudget::new();
        let rounds = (1..)
            .find(|_| {
                finish_a_sleep();
                budget.spend() == Spent::Over
            })
            .expect("ends");
        assert_eq!(rounds, MAX_TOTAL);
    }
}
