//! Motion scalars per level: overshoot, squish, lift, tilt, stagger
//! (design/05-MOTION.md sections 3.1-3.2).

use super::hex::thousandths;
use super::name::VarName;
use crate::appearance::MotionLevel;
use std::time::Duration;

/// One motion scalar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScalarToken {
    /// `--overshoot`: the peak scale inside pop-in, row-in, compose-rise, chip-in, cmdk-in.
    Overshoot,
    /// `--squish`: the pressed scale.
    Squish,
    /// `--lift`: a hovered row's rise.
    Lift,
    /// `--tilt`: the drag ghost's rotation.
    Tilt,
    /// `--stagger`: the delay per list index.
    Stagger,
}

/// A scalar's value, in its own unit, thousandths where it is fractional.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScalarValue {
    /// A unitless factor: 1040 is 1.04.
    Factor(i16),
    /// A length in thousandths of a pixel: -2000 is -2px.
    Length(i32),
    /// An angle in thousandths of a degree: 2200 is 2.2deg.
    Angle(i32),
    /// A time.
    Time(Duration),
}

impl ScalarToken {
    /// Every scalar, in stylesheet order.
    pub const ALL: [ScalarToken; 5] = [
        ScalarToken::Overshoot,
        ScalarToken::Squish,
        ScalarToken::Lift,
        ScalarToken::Tilt,
        ScalarToken::Stagger,
    ];

    /// The custom property: `--overshoot`, …
    pub fn var(self) -> VarName {
        VarName(match self {
            ScalarToken::Overshoot => "--overshoot",
            ScalarToken::Squish => "--squish",
            ScalarToken::Lift => "--lift",
            ScalarToken::Tilt => "--tilt",
            ScalarToken::Stagger => "--stagger",
        })
    }

    /// The value at `level` (Post, design/05-MOTION.md sections 3.1-3.2). Reduced is neutral;
    /// its `--lift` of 0px is proposed (open decision 2).
    pub fn value(self, level: MotionLevel) -> ScalarValue {
        use MotionLevel::{Calm, Extra, Reduced, Standard};
        match (self, level) {
            (ScalarToken::Overshoot, Standard) => ScalarValue::Factor(1040),
            (ScalarToken::Overshoot, Extra) => ScalarValue::Factor(1140),
            (ScalarToken::Overshoot, Calm | Reduced) => ScalarValue::Factor(1000),
            (ScalarToken::Squish, Standard) => ScalarValue::Factor(955),
            (ScalarToken::Squish, Extra) => ScalarValue::Factor(860),
            (ScalarToken::Squish, Calm | Reduced) => ScalarValue::Factor(1000),
            (ScalarToken::Lift, Calm | Standard | Extra) => ScalarValue::Length(-2000),
            (ScalarToken::Lift, Reduced) => ScalarValue::Length(0),
            (ScalarToken::Tilt, Standard) => ScalarValue::Angle(2200),
            (ScalarToken::Tilt, Extra) => ScalarValue::Angle(5000),
            (ScalarToken::Tilt, Calm | Reduced) => ScalarValue::Angle(0),
            (ScalarToken::Stagger, Standard) => ScalarValue::Time(Duration::from_millis(26)),
            (ScalarToken::Stagger, Extra) => ScalarValue::Time(Duration::from_millis(34)),
            (ScalarToken::Stagger, Calm | Reduced) => ScalarValue::Time(Duration::ZERO),
        }
    }
}

impl ScalarValue {
    /// The CSS text: `1.04`, `-2px`, `2.2deg`, `26ms`.
    pub fn css(self) -> String {
        match self {
            ScalarValue::Factor(factor) => thousandths(i64::from(factor)),
            ScalarValue::Length(length) => format!("{}px", thousandths(i64::from(length))),
            ScalarValue::Angle(angle) => format!("{}deg", thousandths(i64::from(angle))),
            ScalarValue::Time(time) => format!("{}ms", time.as_millis()),
        }
    }
}
