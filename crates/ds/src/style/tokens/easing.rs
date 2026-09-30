//! Easing curves per motion level (design/05-MOTION.md sections 3.1-3.4).

use super::hex::thousandths;
use crate::style::appearance::motion::MotionLevel;
use crate::style::tokens::token::{CssValue, Token, TokenScope};
use ds_core::word::Word;

/// A `cubic-bezier()`, control points in thousandths: `(.22,.9,.3,1)` is
/// `[220, 900, 300, 1000]`. Integers, so a curve is `Eq`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CubicBezier(pub [i16; 4]);

/// An easing value as the stylesheet writes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Easing {
    /// `linear`.
    Linear,
    /// A `cubic-bezier(...)`.
    Cubic(CubicBezier),
}

/// One easing token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "e-", kind = fixed, css = easing_css)]
pub enum EasingToken {
    /// `--e-out`: entrances that decelerate.
    Out,
    /// `--e-exit`: exits that accelerate.
    Exit,
    /// `--e-shake`: `shake-x` and C's `shake`.
    Shake,
    /// `--e-linear`: a determinate value, the send ring.
    Linear,
    /// `--e-in-out`: CSS's `ease-in-out`, the default for a state change or a slide.
    InOut,
}

impl EasingToken {
    /// The curve at `level`.
    ///
    /// No easing follows the level: Reduced changes durations, never curves. A spring is a
    /// Rust `Spring`, not an easing.
    pub fn easing(self, _level: MotionLevel) -> Easing {
        const OUT: CubicBezier = CubicBezier([220, 900, 300, 1000]);
        Easing::Cubic(match self {
            EasingToken::Out => OUT,
            EasingToken::Exit => CubicBezier([550, 0, 750, 200]),
            EasingToken::Shake => CubicBezier([360, 70, 190, 970]),
            EasingToken::Linear => return Easing::Linear,
            EasingToken::InOut => CubicBezier([420, 0, 580, 1000]),
        })
    }
}

/// An easing as the stylesheet writes it, at the scope's motion level.
fn easing_css(token: EasingToken, scope: TokenScope) -> CssValue {
    CssValue::computed(token.easing(scope.motion).css())
}

impl Easing {
    /// The CSS text: `linear` or `cubic-bezier(.22,.9,.3,1)`.
    pub fn css(self) -> String {
        match self {
            Easing::Linear => "linear".to_owned(),
            Easing::Cubic(CubicBezier(points)) => {
                let points = points
                    .iter()
                    .map(|&point| thousandths(i64::from(point)))
                    .collect::<Vec<_>>()
                    .join(",");
                format!("cubic-bezier({points})")
            }
        }
    }
}
