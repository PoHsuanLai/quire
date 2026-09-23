//! The Candy shelf as label hues: `--c-{red,amber,green,blue,violet}{,-deep,-soft}`
//! (design/03-COLOR.md section 15, design/07-LOOKS.md section 6.2).
//!
//! Values are `C:24-28` and `C:155-170` verbatim; that they name labels at all is proposed
//! (design/07-LOOKS.md section 11).

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

impl HueMember {
    /// Every member, in the order the stylesheet writes them.
    pub(crate) const ALL: [HueMember; 3] = [HueMember::Base, HueMember::Deep, HueMember::Soft];
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
        match self {
            LabelHue::Red => "red",
            LabelHue::Amber => "amber",
            LabelHue::Green => "green",
            LabelHue::Blue => "blue",
            LabelHue::Violet => "violet",
        }
    }

    /// The custom property for one member: `--c-red-deep`.
    pub fn var(self, member: HueMember) -> String {
        let suffix = match member {
            HueMember::Base => "",
            HueMember::Deep => "-deep",
            HueMember::Soft => "-soft",
        };
        format!("--c-{}{suffix}", self.slug())
    }

    /// The member's value in `scheme`.
    pub fn value(self, member: HueMember, scheme: Scheme) -> Hex {
        let [base, deep, soft] = self.family(scheme);
        let rgb = match member {
            HueMember::Base => base,
            HueMember::Deep => deep,
            HueMember::Soft => soft,
        };
        Hex([(rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8])
    }

    /// Base, deep and soft as `0xRRGGBB` (design/03-COLOR.md section 15).
    fn family(self, scheme: Scheme) -> [u32; 3] {
        match (self, scheme) {
            (LabelHue::Red, Scheme::Light) => [0xE8483C, 0xB7352B, 0xFBE3E1],
            (LabelHue::Amber, Scheme::Light) => [0xF0A81E, 0x8E5A05, 0xFAEBD2],
            (LabelHue::Green, Scheme::Light) => [0x28B24A, 0x1A7A33, 0xDCF1E1],
            (LabelHue::Blue, Scheme::Light) => [0x2B7CFF, 0x0B5FE0, 0xE2EBFF],
            (LabelHue::Violet, Scheme::Light) => [0x8B5CF0, 0x6B3FCC, 0xECE4FB],
            (LabelHue::Red, Scheme::Dark) => [0xFF6B60, 0xFF8A80, 0x3A1E1C],
            (LabelHue::Amber, Scheme::Dark) => [0xF5BC4E, 0xF0B84A, 0x3A2C12],
            (LabelHue::Green, Scheme::Dark) => [0x4FD06A, 0x6EDB86, 0x153520],
            (LabelHue::Blue, Scheme::Dark) => [0x4C9BFF, 0x6FB0FF, 0x14263F],
            (LabelHue::Violet, Scheme::Dark) => [0xA98BFF, 0xB69BFF, 0x271C42],
        }
    }
}
