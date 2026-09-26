//! A value moving from one number to another along an easing curve, as data: what a Rust-driven
//! detail draws at a given time (design/26-DETAILS.md section 3.2, "Why Rust tweens").

use crate::components::Fraction;
use crate::tokens::Easing;
use std::time::Duration;

/// One frame at 60 Hz: how often a moving glide asks for a frame, and never at rest (R3).
pub(crate) const FRAME: Duration = Duration::from_millis(16);

/// A value gliding from `from` to `to` over `length` along `easing`. Values are in whatever unit
/// the caller glides (thousandths of a share, a count); a spring may overshoot `to` on the way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Glide {
    pub(crate) from: i64,
    pub(crate) to: i64,
    pub(crate) length: Duration,
    pub(crate) easing: Easing,
}

/// Where a glide is at one instant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct Pose {
    /// The value now.
    pub(crate) value: i64,
    /// How far along the curve, in thousandths (a spring reads past 1000 on the way).
    pub(crate) eased: Fraction,
    /// How far through its length, in thousandths, not eased.
    pub(crate) through: Fraction,
    /// Time since the glide started.
    pub(crate) elapsed: Duration,
    /// Which of its motor's glides this is: a new glide (or snap) is a new run.
    pub(crate) run: u32,
}

impl Glide {
    /// A glide that is already where it is going.
    pub(crate) fn still(at: i64) -> Glide {
        Glide {
            from: at,
            to: at,
            length: Duration::ZERO,
            easing: Easing::Linear,
        }
    }

    /// The pose `elapsed` after the start: the target once the length has passed.
    pub(crate) fn at(self, elapsed: Duration) -> Pose {
        let through = through(elapsed, self.length);
        let eased = self.easing.at(through);
        let span = self.to - self.from;
        let value = self.from + (span * i64::from(eased.0) + span.signum() * 500) / 1000;
        Pose {
            value: if through.0 >= 1000 { self.to } else { value },
            eased,
            through,
            elapsed,
            run: 0,
        }
    }

    /// Whether the glide has reached its target by `elapsed`.
    pub(crate) fn done(self, elapsed: Duration) -> bool {
        elapsed >= self.length
    }
}

/// `elapsed` as thousandths of `length`, 1000 at and past the end (and for no length at all).
fn through(elapsed: Duration, length: Duration) -> Fraction {
    if length.is_zero() || elapsed >= length {
        return Fraction(1000);
    }
    let share = elapsed.as_micros() * 1000 / length.as_micros();
    Fraction(u16::try_from(share).unwrap_or(1000))
}

#[cfg(test)]
mod tests {
    use super::Glide;
    use crate::appearance::MotionLevel;
    use crate::tokens::{Easing, EasingToken};
    use std::time::Duration;

    const MS: fn(u64) -> Duration = Duration::from_millis;

    #[test]
    fn a_glide_moves_along_its_curve_and_lands_on_its_target() {
        let glide = Glide {
            from: 0,
            to: 930,
            length: MS(700),
            easing: Easing::Linear,
        };
        const CASES: &[(u64, i64)] = &[(0, 0), (350, 465), (700, 930), (5_000, 930)];
        for &(ms, want) in CASES {
            assert_eq!(glide.at(MS(ms)).value, want, "{ms} ms");
        }
        assert!(!glide.done(MS(699)));
        assert!(glide.done(MS(700)));
    }

    #[test]
    fn a_glide_down_and_a_still_glide() {
        let down = Glide {
            from: 800,
            to: 200,
            length: MS(100),
            easing: EasingToken::Out.easing(MotionLevel::Standard),
        };
        let mid = down.at(MS(50)).value;
        assert!((200..800).contains(&mid), "{mid}");
        assert_eq!(down.at(MS(100)).value, 200);
        assert_eq!(Glide::still(42).at(Duration::ZERO).value, 42);
    }

    #[test]
    fn a_spring_overshoots_on_the_way() {
        let spring = Glide {
            from: 0,
            to: 1000,
            length: MS(250),
            easing: EasingToken::Spring.easing(MotionLevel::Standard),
        };
        let peak = (0..250).map(|ms| spring.at(MS(ms)).value).max();
        assert!(peak.is_some_and(|p| p > 1000), "{peak:?}");
    }
}
