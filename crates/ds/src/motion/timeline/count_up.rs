//! CountUp: an integer counting to a new value along an ease, so a number lands with the arc it
//! is drawn beside (design/26-DETAILS.md section 3.2). Named so because `ds::Count` is the count
//! badge. The hook is [`crate::motion::detail::count_up::use_count_up`].

use super::Timeline;
use super::ease::Ease;
use std::time::Duration;

/// A count from `from` to `to` along `ease`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CountUp {
    /// The number the count starts on.
    pub from: i64,
    /// The number it lands on.
    pub to: i64,
    /// The move it follows: the same curve and length as the arc it is in step with.
    pub ease: Ease,
}

impl Timeline for CountUp {
    type Frame = i64;

    fn total(&self) -> Duration {
        self.ease.total()
    }

    /// The number to print: rounded along the curve, never past the target on the way (a spring
    /// curve overshoots, a count does not), exactly the target once the move has landed.
    fn at(&self, elapsed: Duration) -> i64 {
        let along = self.ease.at(elapsed);
        if along.through.0 >= 1000 {
            return self.to;
        }
        let span = self.to - self.from;
        let counted = self.from + (span * i64::from(along.eased.0) + span.signum() * 500) / 1000;
        if span >= 0 {
            counted.min(self.to)
        } else {
            counted.max(self.to)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CountUp, Ease, Timeline};
    use crate::style::appearance::motion::MotionLevel;
    use crate::style::tokens::easing::{Easing, EasingToken};
    use std::time::Duration;

    const MS: fn(u64) -> Duration = Duration::from_millis;

    #[test]
    fn a_count_is_its_start_rounded_along_the_curve_and_its_target_at_the_end() {
        let ease = Ease {
            length: MS(700),
            easing: Easing::Linear,
        };
        // (from, to, the number at zero, half the total and the total).
        const CASES: &[(i64, i64, [i64; 3])] = &[
            (0, 93, [0, 47, 93]),
            (93, 0, [93, 46, 0]),
            (40, 40, [40, 40, 40]),
        ];
        for &(from, to, want) in CASES {
            let count = CountUp { from, to, ease };
            let total = count.total();
            let got = [Duration::ZERO, total / 2, total].map(|at| count.at(at));
            assert_eq!(got, want, "{from} -> {to}");
        }
    }

    #[test]
    fn a_count_never_passes_its_target_on_a_curve_that_overshoots() {
        let count = CountUp {
            from: 0,
            to: 100,
            ease: Ease {
                length: MS(250),
                easing: EasingToken::Spring.easing(MotionLevel::Standard),
            },
        };
        let peak = (0..250).map(|ms| count.at(MS(ms))).max();
        assert_eq!(peak, Some(100));
    }
}
