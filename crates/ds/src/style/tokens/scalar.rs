//! Motion scalars per level: overshoot, squish, lift, tilt, stagger
//! (design/05-MOTION.md sections 3.1-3.2).

use super::hex::thousandths;
use crate::core::word::Word;
use crate::style::appearance::motion::MotionLevel;
use crate::style::tokens::token::{CssValue, Token, TokenScope};
use std::time::Duration;

/// One motion scalar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "", kind = fixed, css = scalar_css)]
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
    /// `--pickup`: the scale of a desktop widget picked up to be moved (design/23
    /// section 9.8), and of a card lifted in the widget gallery.
    Pickup,
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
    /// The value at `level` (design/05-MOTION.md sections 3.1-3.2). Reduced is neutral.
    pub fn value(self, level: MotionLevel) -> ScalarValue {
        use MotionLevel::{Reduced, Standard};
        match (self, level) {
            (ScalarToken::Overshoot, Standard) => ScalarValue::Factor(1040),
            (ScalarToken::Overshoot, Reduced) => ScalarValue::Factor(1000),
            (ScalarToken::Squish, Standard) => ScalarValue::Factor(955),
            (ScalarToken::Squish, Reduced) => ScalarValue::Factor(1000),
            (ScalarToken::Lift, Standard) => ScalarValue::Length(-2000),
            (ScalarToken::Lift, Reduced) => ScalarValue::Length(0),
            (ScalarToken::Tilt, Standard) => ScalarValue::Angle(2200),
            (ScalarToken::Tilt, Reduced) => ScalarValue::Angle(0),
            (ScalarToken::Stagger, Standard) => ScalarValue::Time(Duration::from_millis(26)),
            (ScalarToken::Stagger, Reduced) => ScalarValue::Time(Duration::ZERO),
            (ScalarToken::Pickup, Standard) => ScalarValue::Factor(1040),
            (ScalarToken::Pickup, Reduced) => ScalarValue::Factor(1000),
        }
    }
}

/// A scalar as the stylesheet writes it, at the scope's motion level.
fn scalar_css(token: ScalarToken, scope: TokenScope) -> CssValue {
    CssValue::computed(token.value(scope.motion).css())
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
