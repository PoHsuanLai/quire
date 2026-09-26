//! Settle: how a success lands (design/26-DETAILS.md section 3.2).

use crate::components::Fraction;
use crate::components::vocab::PulseKey;

/// How a success lands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettleStyle {
    /// A level glyph fills cumulatively once to its true value, one layer per `--t-pending-step`.
    Fill(super::pending::Layers),
    /// A check draws on over `--t-move`, holds `SettleHold`, and goes.
    Check,
    /// The element seals: `gulp`'s shape once, the spring only on `Touch::Contact` (R5).
    LockIn,
}

/// What a settling success draws this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Settling {
    /// Nothing settling: draw the state as it is.
    Rest,
    /// Fill: layers `..=n` lit.
    Filling(u8),
    /// Check: the stroke drawn so far, in thousandths (write it as `stroke-dashoffset`).
    Drawing(Fraction),
    /// LockIn: the pulse to render on the sealing element.
    Sealing(PulseKey),
}

impl Settling {
    /// The `data-settle` word: `rest`, `fill`, `check` or `seal`.
    pub fn slug(self) -> &'static str {
        match self {
            Settling::Rest => "rest",
            Settling::Filling(_) => "fill",
            Settling::Drawing(_) => "check",
            Settling::Sealing(_) => "seal",
        }
    }
}
