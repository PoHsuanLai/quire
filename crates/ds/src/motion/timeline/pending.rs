//! A bounded pending loop as a timeline (design/26-DETAILS.md section 3.2, R4): which frame an
//! operation that has run for a given time shows. The hook is
//! [`crate::motion::detail::use_pending::use_pending`].

use super::Timeline;
use crate::motion::detail::operation::Deadline;
use crate::motion::detail::pending::PendingFrame;
use crate::style::appearance::motion::MotionLevel;
use crate::style::tokens::{delay::DelayToken, timing::DurationToken};
use std::time::Duration;

/// An operation's pending loop, one frame at a time: from the operation's age `from` to the
/// instant the frame after it is due.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pending {
    /// No operation: the state as it is.
    Idle,
    /// An operation `from` old, whose loop holds still at `deadline`, played at `level`.
    Running {
        /// When the loop holds still, from the operation's start.
        deadline: Deadline,
        /// The motion level it plays at.
        level: MotionLevel,
        /// How long the operation had already run when this timeline began.
        from: Duration,
    },
}

impl Timeline for Pending {
    type Frame = PendingFrame;

    /// How long until the frame after the one at `from` is due; nothing once the loop holds
    /// still for good (at its deadline, or after its grace under Reduced motion).
    fn total(&self) -> Duration {
        match *self {
            Pending::Idle => Duration::ZERO,
            Pending::Running {
                deadline,
                level,
                from,
            } => next_due(from, deadline, level).unwrap_or(Duration::ZERO),
        }
    }

    /// `Idle` until the operation has run for `PendingGrace`, then a step every
    /// `--t-pending-step`, then `Stalled` from the deadline (or at once after the grace under
    /// Reduced) for as long as the operation still runs.
    fn at(&self, elapsed: Duration) -> PendingFrame {
        match *self {
            Pending::Idle => PendingFrame::Idle,
            Pending::Running {
                deadline,
                level,
                from,
            } => frame_at(from + elapsed, deadline, level),
        }
    }
}

/// The frame an operation `elapsed` into its run shows at `level`, stopping at `deadline`.
fn frame_at(elapsed: Duration, deadline: Deadline, level: MotionLevel) -> PendingFrame {
    let grace = DelayToken::PendingGrace.delay();
    if elapsed < grace {
        return PendingFrame::Idle;
    }
    if level == MotionLevel::Reduced || elapsed >= deadline.length() {
        return PendingFrame::Stalled;
    }
    let step = DurationToken::PendingStep
        .duration(level)
        .as_millis()
        .max(1);
    let n = (elapsed - grace).as_millis() / step;
    PendingFrame::Step(u8::try_from(n).unwrap_or(u8::MAX))
}

/// How long until the frame after the one at `age` is due, or `None` once the loop holds still
/// for good.
fn next_due(age: Duration, deadline: Deadline, level: MotionLevel) -> Option<Duration> {
    let grace = DelayToken::PendingGrace.delay();
    if age < grace {
        return Some(grace - age);
    }
    if level == MotionLevel::Reduced || age >= deadline.length() {
        return None;
    }
    let step = DurationToken::PendingStep.duration(level);
    let into = Duration::from_millis(
        u64::try_from((age - grace).as_millis() % step.as_millis().max(1)).unwrap_or(0),
    );
    Some((step - into).min(deadline.length() - age))
}

#[cfg(test)]
mod tests {
    use super::{Pending, Timeline};
    use crate::motion::detail::operation::Deadline;
    use crate::motion::detail::pending::PendingFrame;
    use crate::style::appearance::motion::MotionLevel;
    use std::time::Duration;

    const MS: fn(u64) -> Duration = Duration::from_millis;

    fn running(deadline: Deadline, level: MotionLevel, from: u64) -> Pending {
        Pending::Running {
            deadline,
            level,
            from: MS(from),
        }
    }

    #[test]
    fn a_loop_waits_for_its_grace_steps_and_holds_at_its_deadline() {
        let cap = Deadline::cap();
        const CASES: &[(u64, MotionLevel, PendingFrame)] = &[
            (0, MotionLevel::Standard, PendingFrame::Idle),
            (399, MotionLevel::Standard, PendingFrame::Idle),
            (400, MotionLevel::Standard, PendingFrame::Step(0)),
            (699, MotionLevel::Standard, PendingFrame::Step(0)),
            (700, MotionLevel::Standard, PendingFrame::Step(1)),
            (9_999, MotionLevel::Standard, PendingFrame::Step(31)),
            (10_000, MotionLevel::Standard, PendingFrame::Stalled),
            (60_000, MotionLevel::Standard, PendingFrame::Stalled),
            (760, MotionLevel::Calm, PendingFrame::Step(1)),
            (399, MotionLevel::Reduced, PendingFrame::Idle),
            (400, MotionLevel::Reduced, PendingFrame::Stalled),
        ];
        for &(ms, level, want) in CASES {
            assert_eq!(running(cap, level, 0).at(MS(ms)), want, "{ms} ms {level:?}");
        }
        assert_eq!(
            running(Deadline::within(MS(1_500)), MotionLevel::Standard, 0).at(MS(2_000)),
            PendingFrame::Stalled
        );
    }

    #[test]
    fn a_segment_runs_from_its_age_to_the_next_frame_and_never_past_the_deadline() {
        let cap = Deadline::cap();
        const CASES: &[(u64, MotionLevel, u64)] = &[
            (0, MotionLevel::Standard, 400),
            (400, MotionLevel::Standard, 300),
            (550, MotionLevel::Standard, 150),
            (9_900, MotionLevel::Standard, 100),
            (10_000, MotionLevel::Standard, 0),
            (0, MotionLevel::Reduced, 400),
            (400, MotionLevel::Reduced, 0),
        ];
        for &(from, level, total) in CASES {
            let segment = running(cap, level, from);
            assert_eq!(segment.total(), MS(total), "{from} ms {level:?}");
            assert!(segment.settled(MS(total)), "{from} ms {level:?}");
            if total > 0 {
                assert!(!segment.settled(MS(total - 1)), "{from} ms {level:?}");
            }
        }
        assert_eq!(Pending::Idle.total(), Duration::ZERO);
        assert_eq!(Pending::Idle.at(MS(5_000)), PendingFrame::Idle);
    }

    #[test]
    fn a_segment_is_at_zero_the_frame_it_began_on_and_at_total_the_next() {
        let segment = running(Deadline::cap(), MotionLevel::Standard, 550);
        let total = segment.total();
        assert_eq!(segment.at(Duration::ZERO), PendingFrame::Step(0));
        assert_eq!(segment.at(total / 2), PendingFrame::Step(0));
        assert_eq!(segment.at(total), PendingFrame::Step(1));
    }
}
