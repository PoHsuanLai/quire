//! A curve over time: how far along its move something is, in thousandths, `elapsed` into a
//! move `length` long (design/26-DETAILS.md section 3.2, "Why Rust tweens").

use super::Timeline;
use crate::core::vocab::Fraction;
use crate::style::tokens::easing::Easing;
use std::time::Duration;

/// A move `length` long along `easing`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Ease {
    pub(crate) length: Duration,
    pub(crate) easing: Easing,
}

/// How far along a move is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct Eased {
    /// How far through its length, in thousandths, not eased.
    pub(crate) through: Fraction,
    /// How far along the curve, in thousandths (a spring curve reads past 1000 on the way).
    pub(crate) eased: Fraction,
}

impl Timeline for Ease {
    type Frame = Eased;

    fn total(&self) -> Duration {
        self.length
    }

    fn at(&self, elapsed: Duration) -> Eased {
        let through = through(elapsed, self.length);
        Eased {
            through,
            eased: self.easing.at(through),
        }
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
    use super::{Ease, Eased, Timeline};
    use crate::core::vocab::Fraction;
    use crate::style::tokens::easing::Easing;
    use std::time::Duration;

    const MS: fn(u64) -> Duration = Duration::from_millis;

    #[test]
    fn a_linear_ease_is_its_share_of_the_length_and_whole_at_the_end() {
        let ease = Ease {
            length: MS(700),
            easing: Easing::Linear,
        };
        assert_eq!(ease.total(), MS(700));
        // (elapsed, through, eased): zero, half, the end and past it.
        const CASES: &[(u64, u16, u16)] = &[
            (0, 0, 0),
            (350, 500, 500),
            (700, 1000, 1000),
            (5_000, 1000, 1000),
        ];
        for &(ms, through, eased) in CASES {
            let want = Eased {
                through: Fraction(through),
                eased: Fraction(eased),
            };
            assert_eq!(ease.at(MS(ms)), want, "{ms} ms");
        }
        assert!(!ease.settled(MS(699)));
        assert!(ease.settled(MS(700)));
    }

    #[test]
    fn an_ease_with_no_length_is_already_whole() {
        let ease = Ease {
            length: Duration::ZERO,
            easing: Easing::Linear,
        };
        assert_eq!(ease.at(Duration::ZERO).through, Fraction(1000));
        assert!(ease.settled(Duration::ZERO));
    }
}
