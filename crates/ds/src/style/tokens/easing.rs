//! Easing curves per motion level (design/05-MOTION.md sections 3.1-3.4).

use super::hex::thousandths;
use crate::core::word::Word;
use crate::style::appearance::motion::MotionLevel;
use crate::style::tokens::token::{CssValue, Token, TokenScope};

/// A `cubic-bezier()`, control points in thousandths: `(.34,1.42,.52,1)` is
/// `[340, 1420, 520, 1000]`. Integers, so a curve is `Eq`.
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
    /// `--e-spring`: overshoot, spent only on contact.
    Spring,
    /// `--e-exit`: exits that accelerate.
    Exit,
    /// `--e-shake`: `shake-x` and C's `shake`.
    Shake,
    /// `--e-linear`: spin, the send ring.
    Linear,
    /// `--e-in-out`: the breathing halo's `ease-in-out` (04-COMPONENTS O-3).
    InOut,
}

impl EasingToken {
    /// The curve at `level`.
    ///
    /// Only the spring follows the level: Reduced uses `--e-out`, so nothing overshoots.
    pub fn easing(self, level: MotionLevel) -> Easing {
        const OUT: CubicBezier = CubicBezier([220, 900, 300, 1000]);
        Easing::Cubic(match (self, level) {
            (EasingToken::Out, _) => OUT,
            (EasingToken::Spring, MotionLevel::Reduced) => OUT,
            (EasingToken::Spring, MotionLevel::Standard) => CubicBezier([340, 1420, 520, 1000]),
            (EasingToken::Exit, _) => CubicBezier([550, 0, 750, 200]),
            (EasingToken::Shake, _) => CubicBezier([360, 70, 190, 970]),
            (EasingToken::Linear, _) => return Easing::Linear,
            // CSS's `ease-in-out`, the halo's curve (`C:288`).
            (EasingToken::InOut, _) => CubicBezier([420, 0, 580, 1000]),
        })
    }
}

/// An easing as the stylesheet writes it, at the scope's motion level.
fn easing_css(token: EasingToken, scope: TokenScope) -> CssValue {
    CssValue::computed(token.easing(scope.motion).css())
}

impl Easing {
    /// The CSS text: `linear` or `cubic-bezier(.34,1.42,.52,1)`.
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
