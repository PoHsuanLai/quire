//! A value moving from one number to another along an ease, as data: what a Rust-driven detail
//! draws at a given time (design/26-DETAILS.md section 3.2, "Why Rust tweens").

use super::Timeline;
use super::ease::Ease;
use crate::style::tokens::easing::Easing;
use ds_core::vocab::Fraction;
use std::time::Duration;

/// A value gliding from `from` to `to` along `ease`. Values are in whatever unit the caller
/// glides (thousandths of a share, a count); a spring curve may overshoot `to` on the way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Glide {
    pub(crate) from: i64,
    pub(crate) to: i64,
    pub(crate) ease: Ease,
}

/// Where a glide is at one instant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Pose {
    /// The value now.
    pub(crate) value: i64,
    /// How far along the curve, in thousandths (a spring curve reads past 1000 on the way).
    pub(crate) eased: Fraction,
    /// How far through its length, in thousandths, not eased.
    pub(crate) through: Fraction,
    /// Time since the glide started.
    pub(crate) elapsed: Duration,
}

impl Glide {
    /// From `from` to `to` over `length` along `easing`.
    pub fn between(from: i64, to: i64, length: Duration, easing: Easing) -> Glide {
        Glide {
            from,
            to,
            ease: Ease { length, easing },
        }
    }

    /// A glide that is already where it is going.
    pub fn still(at: i64) -> Glide {
        Glide::between(at, at, Duration::ZERO, Easing::Linear)
    }
}

impl Timeline for Glide {
    type Frame = Pose;

    fn total(&self) -> Duration {
        self.ease.length
    }

    /// The pose `elapsed` after the start: the target once the length has passed.
    fn at(&self, elapsed: Duration) -> Pose {
        let along = self.ease.at(elapsed);
        let span = self.to - self.from;
        let value = self.from + (span * i64::from(along.eased.0) + span.signum() * 500) / 1000;
        Pose {
            value: if along.through.0 >= 1000 {
                self.to
            } else {
                value
            },
            eased: along.eased,
            through: along.through,
            elapsed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Glide, Timeline};
    use crate::style::appearance::motion::MotionLevel;
    use crate::style::tokens::easing::{Easing, EasingToken};
    use std::time::Duration;

    const MS: fn(u64) -> Duration = Duration::from_millis;

    #[test]
    fn a_glide_moves_along_its_curve_and_lands_on_its_target() {
        let glide = Glide::between(0, 930, MS(700), Easing::Linear);
        assert_eq!(glide.total(), MS(700));
        const CASES: &[(u64, i64)] = &[(0, 0), (350, 465), (700, 930), (5_000, 930)];
        for &(ms, want) in CASES {
            assert_eq!(glide.at(MS(ms)).value, want, "{ms} ms");
        }
        assert!(!glide.settled(MS(699)));
        assert!(glide.settled(MS(700)));
    }

    #[test]
    fn a_glide_down_and_a_still_glide() {
        let down = Glide::between(
            800,
            200,
            MS(100),
            EasingToken::Out.easing(MotionLevel::Standard),
        );
        let mid = down.at(MS(50)).value;
        assert!((200..800).contains(&mid), "{mid}");
        assert_eq!(down.at(MS(100)).value, 200);
        assert_eq!(Glide::still(42).at(Duration::ZERO).value, 42);
    }
}
