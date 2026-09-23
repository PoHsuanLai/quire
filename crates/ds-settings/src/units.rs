//! The units settings keys are written in (design/22-SETTINGS.md section 4, "Shared
//! newtypes"). `Fraction` is `ds`'s own, so a settings gain and a slider value are one type.
//!
//! `Px` here is a whole number of logical pixels, as a key stores it; `ds::Px` is the
//! renderer's fractional layout length.

use serde::{Deserialize, Serialize};

pub use ds::Fraction;

/// Logical pixels, as a key stores them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Px(pub u16);

/// Milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Ms(pub u16);

/// A percentage, 0 to 100, clamped on construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(from = "u8", into = "u8")]
pub struct Percent(pub u8);

impl Percent {
    /// `value`, clamped to 100.
    pub fn new(value: u8) -> Self {
        Percent(value.min(100))
    }
}

impl From<u8> for Percent {
    fn from(value: u8) -> Self {
        Percent::new(value)
    }
}

impl From<Percent> for u8 {
    fn from(value: Percent) -> Self {
        value.0
    }
}

/// A plain quantity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Count(pub u16);

/// A dimensionless physics constant that fits no unit above (momentum model exponents).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Scalar(pub f32);

/// Raw touchpad or report units, signed; device space, not pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Units(pub i32);

#[cfg(test)]
mod tests {
    use super::Percent;

    #[test]
    fn a_percent_is_clamped_on_construction() {
        const CASES: &[(u8, u8)] = &[(0, 0), (80, 80), (100, 100), (101, 100), (255, 100)];
        for &(given, want) in CASES {
            assert_eq!(Percent::new(given), Percent(want), "{given}");
            assert_eq!(Percent::from(given), Percent(want), "from {given}");
        }
    }
}
