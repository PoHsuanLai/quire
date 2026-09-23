//! Easing curves per motion level (design/05-MOTION.md sections 3.1-3.4).
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

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
        todo!()
    }

    /// The curve at `level`.
    pub fn easing(self, level: MotionLevel) -> Easing {
        todo!()
    }
}

impl Easing {
    /// The CSS text: `linear` or `cubic-bezier(.34,1.42,.52,1)`.
    pub fn css(self) -> String {
        todo!()
    }
}
