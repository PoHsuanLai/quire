//! The Candy shelf as label hues: `--c-{red,amber,green,blue,violet}{,-deep,-soft}`
//! (design/03-COLOR.md section 15, design/07-LOOKS.md section 6.2).
//!
//! Values are `C:24-28` and `C:155-170` verbatim; that they name labels at all is proposed
//! (design/07-LOOKS.md section 11).

use super::hex::Hex;
use super::token::{CssValue, Token, TokenScope};
use crate::style::appearance::theme::Scheme;
use ds_core::word::Word;
use serde::{Deserialize, Serialize};

/// One label hue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Word)]
#[serde(rename_all = "snake_case")]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum HueMember {
    /// `--c-<hue>`.
    Base,
    /// `--c-<hue>-deep`: text-safe on the soft tint.
    Deep,
    /// `--c-<hue>-soft`: the chip ground.
    Soft,
}

/// One label hue's member as a token: `--c-red`, `--c-red-deep`, `--c-red-soft`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "c-", kind = fixed, css = hue_css)]
pub enum HueColour {
    /// `--c-red`.
    Red,
    /// `--c-red-deep`.
    RedDeep,
    /// `--c-red-soft`.
    RedSoft,
    /// `--c-amber`.
    Amber,
    /// `--c-amber-deep`.
    AmberDeep,
    /// `--c-amber-soft`.
    AmberSoft,
    /// `--c-green`.
    Green,
    /// `--c-green-deep`.
    GreenDeep,
    /// `--c-green-soft`.
    GreenSoft,
    /// `--c-blue`.
    Blue,
    /// `--c-blue-deep`.
    BlueDeep,
    /// `--c-blue-soft`.
    BlueSoft,
    /// `--c-violet`.
    Violet,
    /// `--c-violet-deep`.
    VioletDeep,
    /// `--c-violet-soft`.
    VioletSoft,
}

impl HueColour {
    /// The hue and member this token is: the family in its order, each hue's three members.
    pub fn parts(self) -> (LabelHue, HueMember) {
        let index = Self::ALL
            .iter()
            .position(|token| *token == self)
            .unwrap_or(0);
        let members = HueMember::ALL.len();
        (
            LabelHue::ALL[index / members],
            HueMember::ALL[index % members],
        )
    }
}

impl LabelHue {
    /// The token for one member: `--c-red-deep`.
    pub fn colour(self, member: HueMember) -> HueColour {
        let hue = LabelHue::ALL
            .iter()
            .position(|hue| *hue == self)
            .unwrap_or(0);
        let member = HueMember::ALL
            .iter()
            .position(|m| *m == member)
            .unwrap_or(0);
        HueColour::ALL[hue * HueMember::ALL.len() + member]
    }
}

/// A hue member as the stylesheet writes it, in the scope's scheme.
fn hue_css(token: HueColour, scope: TokenScope) -> CssValue {
    let (hue, member) = token.parts();
    CssValue::computed(hue.value(member, scope.scheme).css())
}

impl LabelHue {
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
