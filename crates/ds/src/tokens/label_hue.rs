//! The Candy shelf as label hues: `--c-{red,amber,green,blue,violet}{,-deep,-soft}`
//! (design/03-COLOR.md section 15, design/07-LOOKS.md section 6.2).
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

use super::hex::Hex;
use crate::appearance::Scheme;

/// One label hue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LabelHue {
    /// Red.
    Red,
    /// Amber.
    Amber,
    /// Green.
    Green,
    /// Blue.
    Blue,
    /// Violet.
    Violet,
}

/// Which member of a hue's family: the base, the text-safe deep, or the soft tint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HueMember {
    /// `--c-<hue>`.
    Base,
    /// `--c-<hue>-deep`: text-safe on the soft tint.
    Deep,
    /// `--c-<hue>-soft`: the chip ground.
    Soft,
}

impl LabelHue {
    /// Every hue, in the shelf's order.
    pub const ALL: [LabelHue; 5] = [
        LabelHue::Red,
        LabelHue::Amber,
        LabelHue::Green,
        LabelHue::Blue,
        LabelHue::Violet,
    ];

    /// The `data-hue` word: `red`, `amber`, …
    pub fn slug(self) -> &'static str {
        todo!()
    }

    /// The custom property for one member: `--c-red-deep`.
    pub fn var(self, member: HueMember) -> String {
        todo!()
    }

    /// The member's value in `scheme`.
    pub fn value(self, member: HueMember, scheme: Scheme) -> Hex {
        todo!()
    }
}
