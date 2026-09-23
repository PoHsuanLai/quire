//! Motion scalars per level: overshoot, squish, lift, tilt, stagger
//! (design/05-MOTION.md sections 3.1-3.2).
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

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
        todo!()
    }

    /// The value at `level`.
    pub fn value(self, level: MotionLevel) -> ScalarValue {
        todo!()
    }
}

impl ScalarValue {
    /// The CSS text: `1.04`, `-2px`, `2.2deg`, `26ms`.
    pub fn css(self) -> String {
        todo!()
    }
}
