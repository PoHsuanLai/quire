//! The plate gradient families (design/08-ICONS.md section 2.3): two stops at 135 degrees from
//! the Candy shelf, and the glyph colour each takes. A plate is the whole `n = 5` superellipse
//! (`plate.rs`) filled with its family's gradient, with the section 2.5 inner highlight and rim
//! and a drop shadow; until the generated app icons arrive, a placeholder tile is a glyph on one.

use crate::tokens::Hex;
use crate::tokens::VarName;
use crate::tokens::tuned::Tuned;

/// One plate gradient family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlateFamily {
    /// `#E8483C` to `#B7352B`, white glyph.
    Red,
    /// `#F0A81E` to `#8E5A05`, ink glyph.
    Amber,
    /// `#28B24A` to `#1A7A33`, ink glyph.
    Green,
    /// `#2B7CFF` to `#0B5FE0`, white glyph.
    Blue,
    /// `#8B5CF0` to `#6B3FCC`, white glyph.
    Violet,
    /// Paper, `#FFFFFF` to `#F1F3EE` (dark `#2A2E28` to `#1D211B`), the ink glyph: third-party
    /// icons and the symbolic fallback (section 4.1).
    Neutral,
}

impl PlateFamily {
    /// Every family, in section 2.3's order.
    pub const ALL: [PlateFamily; 6] = [
        PlateFamily::Red,
        PlateFamily::Amber,
        PlateFamily::Green,
        PlateFamily::Blue,
        PlateFamily::Violet,
        PlateFamily::Neutral,
    ];

    /// The `data-family` word.
    pub fn slug(self) -> &'static str {
        match self {
            PlateFamily::Red => "red",
            PlateFamily::Amber => "amber",
            PlateFamily::Green => "green",
            PlateFamily::Blue => "blue",
            PlateFamily::Violet => "violet",
            PlateFamily::Neutral => "neutral",
        }
    }

    /// The light stop and the deep stop.
    pub fn stops(self) -> (Hex, Hex) {
        match self {
            PlateFamily::Red => (Hex([0xE8, 0x48, 0x3C]), Hex([0xB7, 0x35, 0x2B])),
            PlateFamily::Amber => (Hex([0xF0, 0xA8, 0x1E]), Hex([0x8E, 0x5A, 0x05])),
            PlateFamily::Green => (Hex([0x28, 0xB2, 0x4A]), Hex([0x1A, 0x7A, 0x33])),
            PlateFamily::Blue => (Hex([0x2B, 0x7C, 0xFF]), Hex([0x0B, 0x5F, 0xE0])),
            PlateFamily::Violet => (Hex([0x8B, 0x5C, 0xF0]), Hex([0x6B, 0x3F, 0xCC])),
            PlateFamily::Neutral => (Hex([0xFF, 0xFF, 0xFF]), Hex([0xF1, 0xF3, 0xEE])),
        }
    }

    /// The glyph colour on the plate (section 2.3's WCAG-driven column).
    pub fn glyph(self) -> Hex {
        match self {
            PlateFamily::Red | PlateFamily::Blue | PlateFamily::Violet => Hex([255, 255, 255]),
            PlateFamily::Amber | PlateFamily::Green => Hex([0x16, 0x17, 0x1A]),
            PlateFamily::Neutral => Hex([0x1A, 0x1E, 0x1A]),
        }
    }
}

/// The neutral plate in the dark scheme (section 4.1): `#2A2E28` to `#1D211B`, and its glyph.
pub const NEUTRAL_DARK: (Hex, Hex, Hex) = (
    Hex([0x2A, 0x2E, 0x28]),
    Hex([0x1D, 0x21, 0x1B]),
    Hex([0xE8, 0xEA, 0xE4]),
);

/// `--plate-glyph`: a glyph's share of the plate (`icons.symbolic_fallback_glyph_percent`, 56).
pub const PLATE_GLYPH: Tuned = Tuned {
    token: VarName("--plate-glyph"),
    input: VarName("--icons-glyph-share"),
    default: "56%",
};

/// `--plate-inset`: a third-party icon's share of the plate (`icons.plate_inset_percent`, 72).
pub const PLATE_INSET: Tuned = Tuned {
    token: VarName("--plate-inset"),
    input: VarName("--icons-inset-share"),
    default: "72%",
};

#[cfg(test)]
mod tests {
    use super::PlateFamily;

    #[test]
    fn white_glyphs_sit_on_red_blue_and_violet_and_ink_on_the_rest() {
        let white = [PlateFamily::Red, PlateFamily::Blue, PlateFamily::Violet];
        for family in PlateFamily::ALL {
            assert_eq!(
                family.glyph().0 == [255, 255, 255],
                white.contains(&family),
                "{family:?}"
            );
        }
    }
}
