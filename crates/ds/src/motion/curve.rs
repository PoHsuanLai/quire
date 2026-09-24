//! Evaluating an easing token at a point in time, for motion driven from a frame clock rather
//! than by CSS: a dock's magnification progress, an auto-hide slide (design/05-MOTION.md
//! section 3, sill FINDINGS Q2).
//!
//! Integer arithmetic throughout, so the same token at the same time is the same value on every
//! machine and the result is `Eq`: the curve parameter is found by bisection on x in 1/2^24
//! steps (x is monotonic because CSS keeps `x1, x2` in 0..=1), and y is read at it.

use crate::components::Fraction;
use crate::tokens::{CubicBezier, Easing};

/// Bisection steps: the parameter's resolution is 1/2^24, far below one thousandth.
const STEPS: u32 = 24;
/// The parameter's scale: `s = n / SCALE`.
const SCALE: i128 = 1 << STEPS;
/// One whole, in the thousandths control points and results are written in.
const WHOLE: i128 = 1000;

impl CubicBezier {
    /// The curve's progress at time `t`, both in thousandths. `t` past 1000 is 1000; a spring's
    /// overshoot reads above 1000, and a value below 0 (a curve CSS allows but no token uses)
    /// reads as 0.
    pub fn at(self, t: Fraction) -> Fraction {
        let [x1, y1, x2, y2] = self.0.map(i128::from);
        let t = i128::from(t.0).min(WHOLE);
        let n = solve_x(x1.clamp(0, WHOLE), x2.clamp(0, WHOLE), t);
        let y = coordinate(y1, y2, n);
        let cube = SCALE * SCALE * SCALE;
        let rounded = (y + cube / 2).div_euclid(cube);
        Fraction(u16::try_from(rounded.max(0)).unwrap_or(u16::MAX))
    }
}

impl Easing {
    /// The curve this easing is: `linear` is the identity Bézier `(0, 0, 1, 1)`.
    pub fn curve(self) -> CubicBezier {
        match self {
            Easing::Linear => CubicBezier([0, 0, 1000, 1000]),
            Easing::Cubic(curve) => curve,
        }
    }

    /// The easing's progress at time `t`, both in thousandths ([`CubicBezier::at`]).
    pub fn at(self, t: Fraction) -> Fraction {
        self.curve().at(t)
    }
}

/// One coordinate of the unit Bézier with inner control values `p1`, `p2` (thousandths) at
/// parameter `n / SCALE`, scaled by `SCALE^3` (so thousandths times `SCALE^3`).
fn coordinate(p1: i128, p2: i128, n: i128) -> i128 {
    let m = SCALE - n;
    3 * m * m * n * p1 + 3 * m * n * n * p2 + WHOLE * n * n * n
}

/// The smallest parameter step `n` whose x reaches `t` (thousandths).
fn solve_x(x1: i128, x2: i128, t: i128) -> i128 {
    let target = t * SCALE * SCALE * SCALE;
    let (mut lo, mut hi) = (0, SCALE);
    while hi - lo > 1 {
        let mid = (lo + hi) / 2;
        if coordinate(x1, x2, mid) < target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    if coordinate(x1, x2, lo) >= target {
        lo
    } else {
        hi
    }
}

#[cfg(test)]
mod tests {
    use crate::appearance::MotionLevel;
    use crate::components::Fraction;
    use crate::tokens::{CubicBezier, Easing, EasingToken};

    const QUARTERS: [u16; 5] = [0, 250, 500, 750, 1000];

    /// Each token's value at 0, .25, .5, .75 and 1, computed by exact rational bisection
    /// (Python `fractions`, 80 steps) and rounded to the thousandth.
    const CASES: &[(&str, EasingToken, MotionLevel, [u16; 5])] = &[
        (
            "out",
            EasingToken::Out,
            MotionLevel::Standard,
            [0, 748, 949, 993, 1000],
        ),
        (
            "spring",
            EasingToken::Spring,
            MotionLevel::Standard,
            [0, 793, 1051, 1031, 1000],
        ),
        (
            "spring extra",
            EasingToken::Spring,
            MotionLevel::Extra,
            [0, 1051, 1238, 1080, 1000],
        ),
        (
            "spring calm is out",
            EasingToken::Spring,
            MotionLevel::Calm,
            [0, 748, 949, 993, 1000],
        ),
        (
            "exit",
            EasingToken::Exit,
            MotionLevel::Standard,
            [0, 19, 110, 382, 1000],
        ),
        (
            "shake",
            EasingToken::Shake,
            MotionLevel::Standard,
            [0, 311, 790, 955, 1000],
        ),
        (
            "linear",
            EasingToken::Linear,
            MotionLevel::Standard,
            [0, 250, 500, 750, 1000],
        ),
        (
            "in-out",
            EasingToken::InOut,
            MotionLevel::Standard,
            [0, 129, 500, 871, 1000],
        ),
    ];

    #[test]
    fn every_token_reads_its_known_values_at_the_quarters() {
        for &(name, token, level, want) in CASES {
            let easing = token.easing(level);
            let got = QUARTERS.map(|t| easing.at(Fraction(t)).0);
            assert_eq!(got, want, "{name}");
        }
    }

    #[test]
    fn a_curve_whose_y_stays_in_range_never_goes_back() {
        for token in EasingToken::ALL {
            for level in [MotionLevel::Calm, MotionLevel::Standard, MotionLevel::Extra] {
                let curve = token.easing(level).curve();
                let [_, y1, _, y2] = curve.0;
                if !(0..=1000).contains(&y1) || !(0..=1000).contains(&y2) {
                    continue;
                }
                let mut last = 0;
                for t in 0..=1000 {
                    let now = curve.at(Fraction(t)).0;
                    assert!(now >= last, "{token:?} {level:?} at {t}: {now} < {last}");
                    last = now;
                }
            }
        }
    }

    #[test]
    fn a_spring_overshoots_and_settles_on_one() {
        let spring = EasingToken::Spring.easing(MotionLevel::Standard);
        let peak = (0..=1000).map(|t| spring.at(Fraction(t)).0).max();
        assert!(peak.is_some_and(|p| p > 1000), "{peak:?}");
        assert_eq!(spring.at(Fraction(1000)), Fraction(1000));
    }

    #[test]
    fn time_past_the_end_clamps_and_linear_is_the_identity() {
        assert_eq!(
            EasingToken::Out
                .easing(MotionLevel::Standard)
                .at(Fraction(4000)),
            Fraction(1000)
        );
        for t in 0..=1000 {
            assert_eq!(Easing::Linear.at(Fraction(t)), Fraction(t), "{t}");
        }
        assert_eq!(Easing::Linear.curve(), CubicBezier([0, 0, 1000, 1000]));
    }

    #[test]
    fn a_curve_that_dips_below_zero_reads_zero() {
        // CSS's `back-in`-like curve: y1 negative.
        let dip = CubicBezier([600, -280, 735, 45]);
        assert_eq!(dip.at(Fraction(200)), Fraction(0));
    }
}
