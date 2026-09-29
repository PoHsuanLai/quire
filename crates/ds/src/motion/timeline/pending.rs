//! A pending loop as a timeline (design/26-DETAILS.md section 3.2, R4; design/30 section 1.3):
//! which frame an operation that has run for a given time shows. It spins at once, in steps of
//! `--t-spin-step`, for as long as the operation runs, and keeps turning under Reduced motion
//! (the step is a hold token). The hook is [`crate::motion::detail::use_pending::use_pending`].

use super::Timeline;
use crate::motion::detail::pending::{PendingFrame, SPIN_STEPS};
use crate::style::tokens::timing::DurationToken;
use std::time::Duration;

/// An operation's pending loop, one frame at a time: from the operation's age `from` to the
/// instant the frame after it is due.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pending {
    /// No operation: the state as it is.
    Idle,
    /// An operation `from` old.
    Running {
        /// How long the operation had already run when this timeline began.
        from: Duration,
    },
}

/// One step of the loop, the same at every motion level.
fn step() -> Duration {
    DurationToken::SpinStep.duration(crate::style::appearance::motion::MotionLevel::Standard)
}

impl Timeline for Pending {
    type Frame = PendingFrame;

    /// How long until the frame after the one at `from` is due.
    fn total(&self) -> Duration {
        match *self {
            Pending::Idle => Duration::ZERO,
            Pending::Running { from } => {
                let step = step();
                let into = Duration::from_millis(
                    u64::try_from(from.as_millis() % step.as_millis().max(1)).unwrap_or(0),
                );
                step - into
            }
        }
    }

    /// `Idle`, or the step the operation's age falls in.
    fn at(&self, elapsed: Duration) -> PendingFrame {
        match *self {
            Pending::Idle => PendingFrame::Idle,
            Pending::Running { from } => {
                let n = (from + elapsed).as_millis() / step().as_millis().max(1);
                PendingFrame::Step(u8::try_from(n % u128::from(SPIN_STEPS)).unwrap_or(0))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Pending, Timeline};
    use crate::motion::detail::pending::PendingFrame;
    use std::time::Duration;

    const MS: fn(u64) -> Duration = Duration::from_millis;

    #[test]
    fn a_loop_steps_at_once_and_goes_round() {
        const CASES: &[(u64, u8)] = &[(0, 0), (82, 0), (83, 1), (166, 2), (995, 11), (996, 0)];
        for &(ms, step) in CASES {
            assert_eq!(
                Pending::Running { from: MS(0) }.at(MS(ms)),
                PendingFrame::Step(step),
                "{ms} ms"
            );
        }
        assert_eq!(Pending::Idle.at(MS(5_000)), PendingFrame::Idle);
    }

    #[test]
    fn a_segment_runs_from_its_age_to_the_next_frame() {
        const CASES: &[(u64, u64)] = &[(0, 83), (40, 43), (83, 83), (100, 66)];
        for &(from, total) in CASES {
            let segment = Pending::Running { from: MS(from) };
            assert_eq!(segment.total(), MS(total), "{from} ms");
            assert!(segment.settled(MS(total)), "{from} ms");
            assert!(!segment.settled(MS(total - 1)), "{from} ms");
        }
        assert_eq!(Pending::Idle.total(), Duration::ZERO);
    }

    #[test]
    fn a_segment_is_at_zero_the_frame_it_began_on_and_at_total_the_next() {
        let segment = Pending::Running { from: MS(100) };
        assert_eq!(segment.at(Duration::ZERO), PendingFrame::Step(1));
        assert_eq!(segment.at(segment.total()), PendingFrame::Step(2));
    }
}
