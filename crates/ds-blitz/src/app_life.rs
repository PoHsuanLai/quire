//! When the app's event loop ends: the policy the app chooses and the state machine that applies
//! it. The windows are independent (closing one closes only it), so the end of the loop is the
//! app's decision, never a side effect of one window's close. The machine is pure (the caller
//! hands it the time), so its rules are tested without a window.

use std::time::{Duration, Instant};

/// What the app does when its last window closes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LastWindowClosed {
    /// The event loop ends and `launch` returns. A loop that has had no window yet (an app that
    /// opens its first through its [`AppHandle`](crate::AppHandle)) keeps running: nothing has
    /// closed.
    #[default]
    Exit,
    /// The loop keeps running, with no window, for this long after the last close (or after it
    /// started with no window), and ends then unless a window opened. An app that stays warm for
    /// its next launch uses this.
    StayFor(Duration),
}

/// What the loop does next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Verdict {
    /// Keep running.
    Run,
    /// Drop every window and end the loop.
    Exit,
}

/// How many windows are open, and since when there were none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Lifecycle {
    policy: LastWindowClosed,
    open: usize,
    /// How many holds are keeping the loop alive with no window: a program that plays with no
    /// window holds the loop while it does.
    held: usize,
    /// Since when no window has been open, if none is and the policy cares.
    idle_since: Option<Instant>,
    quitting: bool,
}

impl Lifecycle {
    /// A loop that starts at `now` with no window yet.
    pub(crate) fn new(policy: LastWindowClosed, now: Instant) -> Self {
        let idle_since = match policy {
            LastWindowClosed::Exit => None,
            LastWindowClosed::StayFor(_) => Some(now),
        };
        Lifecycle {
            policy,
            open: 0,
            held: 0,
            idle_since,
            quitting: false,
        }
    }

    /// A window opened.
    pub(crate) fn opened(&mut self) {
        self.open += 1;
        self.idle_since = None;
    }

    /// Something other than a window wants the loop to go on: while any hold is taken the loop
    /// does not end, whatever the policy.
    pub(crate) fn hold(&mut self) {
        self.held += 1;
        self.idle_since = None;
    }

    /// A hold was let go, at `now`. With no window open and no hold left, the loop is idle from
    /// now, so the policy's linger starts (or the loop ends, under `Exit`).
    pub(crate) fn release(&mut self, now: Instant) {
        if self.held == 0 {
            // No hold to release: a release that raced one already counted.
            return;
        }
        self.held -= 1;
        if self.held == 0 && self.open == 0 {
            self.idle_since = Some(now);
        }
    }

    /// A window closed, at `now`.
    pub(crate) fn closed(&mut self, now: Instant) {
        if self.open == 0 {
            // No window to close: a close that raced one already counted.
            return;
        }
        self.open -= 1;
        if self.open == 0 && self.held == 0 {
            self.idle_since = Some(now);
        }
    }

    /// The app asked to end the loop.
    pub(crate) fn quit(&mut self) {
        self.quitting = true;
    }

    /// Whether the loop runs on at `now`.
    pub(crate) fn verdict(&self, now: Instant) -> Verdict {
        if self.quitting {
            return Verdict::Exit;
        }
        match (self.policy, self.idle_since) {
            (_, None) => Verdict::Run,
            (LastWindowClosed::Exit, Some(_)) => Verdict::Exit,
            (LastWindowClosed::StayFor(linger), Some(since)) => match since.checked_add(linger) {
                Some(end) if now < end => Verdict::Run,
                Some(_) => Verdict::Exit,
                None => Verdict::Run,
            },
        }
    }

    /// When the loop must look again with nothing else happening: the end of a linger.
    pub(crate) fn wake_at(&self) -> Option<Instant> {
        match (self.policy, self.idle_since) {
            (LastWindowClosed::StayFor(linger), Some(since)) if !self.quitting => {
                since.checked_add(linger)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(start: Instant, ms: u64) -> Instant {
        start + Duration::from_millis(ms)
    }

    #[test]
    fn exit_ends_the_loop_when_the_last_window_closes_and_not_before() {
        let t0 = Instant::now();
        let mut life = Lifecycle::new(LastWindowClosed::Exit, t0);
        assert_eq!(life.verdict(t0), Verdict::Run, "nothing has closed yet");
        life.opened();
        life.opened();
        life.closed(at(t0, 10));
        assert_eq!(life.verdict(at(t0, 10)), Verdict::Run, "one is still open");
        life.closed(at(t0, 20));
        assert_eq!(life.verdict(at(t0, 20)), Verdict::Exit);
    }

    #[test]
    fn closing_the_first_window_leaves_the_others_running() {
        // Which window closes does not matter to the machine: only how many are left.
        let t0 = Instant::now();
        let mut life = Lifecycle::new(LastWindowClosed::Exit, t0);
        life.opened();
        life.opened();
        life.closed(t0);
        assert_eq!(life.verdict(at(t0, 1_000_000)), Verdict::Run);
        assert_eq!(life.wake_at(), None);
    }

    #[test]
    fn stay_for_lingers_after_the_last_close_then_ends() {
        let t0 = Instant::now();
        let linger = Duration::from_millis(500);
        let mut life = Lifecycle::new(LastWindowClosed::StayFor(linger), t0);
        life.opened();
        life.closed(at(t0, 100));
        assert_eq!(life.verdict(at(t0, 599)), Verdict::Run);
        assert_eq!(life.wake_at(), Some(at(t0, 600)));
        assert_eq!(life.verdict(at(t0, 600)), Verdict::Exit);
    }

    #[test]
    fn a_window_opened_while_lingering_cancels_the_end() {
        let t0 = Instant::now();
        let mut life = Lifecycle::new(LastWindowClosed::StayFor(Duration::from_millis(500)), t0);
        life.opened();
        life.closed(at(t0, 100));
        life.opened();
        assert_eq!(life.verdict(at(t0, 10_000)), Verdict::Run);
        assert_eq!(life.wake_at(), None);
        life.closed(at(t0, 10_000));
        assert_eq!(
            life.wake_at(),
            Some(at(t0, 10_500)),
            "the linger starts over"
        );
    }

    #[test]
    fn stay_for_counts_from_the_start_when_no_window_ever_opened() {
        let t0 = Instant::now();
        let life = Lifecycle::new(LastWindowClosed::StayFor(Duration::from_millis(50)), t0);
        assert_eq!(life.verdict(at(t0, 49)), Verdict::Run);
        assert_eq!(life.verdict(at(t0, 50)), Verdict::Exit);
    }

    #[test]
    fn quit_ends_the_loop_with_windows_open_under_either_policy() {
        let t0 = Instant::now();
        for policy in [
            LastWindowClosed::Exit,
            LastWindowClosed::StayFor(Duration::from_secs(600)),
        ] {
            let mut life = Lifecycle::new(policy, t0);
            life.opened();
            life.quit();
            assert_eq!(life.verdict(t0), Verdict::Exit, "{policy:?}");
            assert_eq!(life.wake_at(), None);
        }
    }

    #[test]
    fn a_linger_too_long_to_add_never_ends() {
        let t0 = Instant::now();
        let mut life = Lifecycle::new(LastWindowClosed::StayFor(Duration::MAX), t0);
        life.opened();
        life.closed(t0);
        assert_eq!(life.verdict(at(t0, 1_000_000)), Verdict::Run);
        assert_eq!(life.wake_at(), None);
    }

    #[test]
    fn an_extra_close_never_underflows() {
        let t0 = Instant::now();
        let mut life = Lifecycle::new(LastWindowClosed::Exit, t0);
        life.closed(t0);
        life.opened();
        life.closed(t0);
        assert_eq!(life.verdict(t0), Verdict::Exit);
    }

    #[test]
    fn a_hold_keeps_a_loop_with_no_window_running_and_its_release_starts_the_linger() {
        let t0 = Instant::now();
        let linger = Duration::from_millis(500);
        let mut life = Lifecycle::new(LastWindowClosed::StayFor(linger), t0);
        life.hold();
        assert_eq!(
            life.verdict(at(t0, 10_000)),
            Verdict::Run,
            "held past the linger"
        );
        assert_eq!(life.wake_at(), None);
        life.release(at(t0, 10_000));
        assert_eq!(life.verdict(at(t0, 10_499)), Verdict::Run);
        assert_eq!(life.wake_at(), Some(at(t0, 10_500)));
        assert_eq!(life.verdict(at(t0, 10_500)), Verdict::Exit);
    }

    #[test]
    fn a_hold_outlives_the_last_window_and_two_holds_need_two_releases() {
        let t0 = Instant::now();
        let mut life = Lifecycle::new(LastWindowClosed::Exit, t0);
        life.opened();
        life.hold();
        life.hold();
        life.closed(at(t0, 10));
        assert_eq!(life.verdict(at(t0, 20)), Verdict::Run, "held, no window");
        life.release(at(t0, 30));
        assert_eq!(life.verdict(at(t0, 40)), Verdict::Run, "one hold is left");
        life.release(at(t0, 50));
        assert_eq!(life.verdict(at(t0, 60)), Verdict::Exit, "nothing holds it");
    }

    #[test]
    fn a_release_with_a_window_open_leaves_the_loop_running_and_an_extra_one_changes_nothing() {
        let t0 = Instant::now();
        let mut life = Lifecycle::new(LastWindowClosed::Exit, t0);
        life.release(t0);
        life.opened();
        life.hold();
        life.release(at(t0, 5));
        assert_eq!(life.verdict(at(t0, 6)), Verdict::Run, "a window is open");
    }
}
