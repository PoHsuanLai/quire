//! Easing curves per motion level (design/05-MOTION.md sections 3.1-3.4).

use super::hex::thousandths;
use super::name::VarName;
use crate::appearance::MotionLevel;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    /// Every easing token, in stylesheet order.
    pub const ALL: [EasingToken; 6] = [
        EasingToken::Out,
        EasingToken::Spring,
        EasingToken::Exit,
        EasingToken::Shake,
        EasingToken::Linear,
        EasingToken::InOut,
    ];

    /// The custom property: `--e-out`, …
    pub fn var(self) -> VarName {
        VarName(match self {
            EasingToken::Out => "--e-out",
            EasingToken::Spring => "--e-spring",
            EasingToken::Exit => "--e-exit",
            EasingToken::Shake => "--e-shake",
            EasingToken::Linear => "--e-linear",
            EasingToken::InOut => "--e-in-out",
        })
    }

    /// The curve at `level`.
    ///
    /// Only the spring follows the level (section 3.2): Calm uses `--e-out`, Extra
    /// `(.34,2.0,.5,1)`, and Reduced `--e-out` as Calm does (proposed, open decision 2).
    pub fn easing(self, level: MotionLevel) -> Easing {
        const OUT: CubicBezier = CubicBezier([220, 900, 300, 1000]);
        Easing::Cubic(match (self, level) {
            (EasingToken::Out, _) => OUT,
            (EasingToken::Spring, MotionLevel::Calm | MotionLevel::Reduced) => OUT,
            (EasingToken::Spring, MotionLevel::Standard) => CubicBezier([340, 1420, 520, 1000]),
            (EasingToken::Spring, MotionLevel::Extra) => CubicBezier([340, 2000, 500, 1000]),
            (EasingToken::Exit, _) => CubicBezier([550, 0, 750, 200]),
            (EasingToken::Shake, _) => CubicBezier([360, 70, 190, 970]),
            (EasingToken::Linear, _) => return Easing::Linear,
            // CSS's `ease-in-out`, the halo's curve (`C:288`).
            (EasingToken::InOut, _) => CubicBezier([420, 0, 580, 1000]),
        })
    }
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
