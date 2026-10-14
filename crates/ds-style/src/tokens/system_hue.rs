//! Apple's twelve system colours (Human Interface Guidelines, Color, 2025): each hue in the
//! four forms the platform gives it, light and dark, at ordinary and at increased contrast.
//!
//! The accents, the signal colours and the tag colours all read their hues here, so a hue is
//! written once. The values are Apple's own (`color.json` of the HIG, 2025).

use crate::appearance::{system::Contrast, theme::Scheme};
use crate::tokens::hex::Hex;
use ds_core::word::Word;

/// One of the twelve system hues, in the order the HIG lists them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
#[non_exhaustive]
pub enum SystemHue {
    /// systemRed.
    Red,
    /// systemOrange.
    Orange,
    /// systemYellow.
    Yellow,
    /// systemGreen.
    Green,
    /// systemMint.
    Mint,
    /// systemTeal.
    Teal,
    /// systemCyan.
    Cyan,
    /// systemBlue.
    Blue,
    /// systemIndigo.
    Indigo,
    /// systemPurple.
    Purple,
    /// systemPink.
    Pink,
    /// systemBrown.
    Brown,
}

/// A hue's four values: ordinary and increased contrast, in each scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HueShades {
    /// Light, ordinary contrast.
    pub light: Hex,
    /// Light, increased contrast.
    pub light_increased: Hex,
    /// Dark, ordinary contrast.
    pub dark: Hex,
    /// Dark, increased contrast.
    pub dark_increased: Hex,
}

impl HueShades {
    /// The shade for `scheme` at `contrast`.
    pub const fn pick(self, scheme: Scheme, contrast: Contrast) -> Hex {
        match (scheme, contrast) {
            (Scheme::Light, Contrast::Normal) => self.light,
            (Scheme::Light, Contrast::High) => self.light_increased,
            (Scheme::Dark, Contrast::Normal) => self.dark,
            (Scheme::Dark, Contrast::High) => self.dark_increased,
        }
    }
}

const fn hex(value: u32) -> Hex {
    Hex([(value >> 16) as u8, (value >> 8) as u8, value as u8])
}

const fn shades(light: u32, light_increased: u32, dark: u32, dark_increased: u32) -> HueShades {
    HueShades {
        light: hex(light),
        light_increased: hex(light_increased),
        dark: hex(dark),
        dark_increased: hex(dark_increased),
    }
}

impl SystemHue {
    /// The hue's four values.
    pub const fn shades(self) -> HueShades {
        match self {
            SystemHue::Red => shades(0xFF383C, 0xE9152D, 0xFF4245, 0xFF6165),
            SystemHue::Orange => shades(0xFF8D28, 0xC55300, 0xFF9230, 0xFFA056),
            SystemHue::Yellow => shades(0xFFCC00, 0xA16A00, 0xFFD600, 0xFEDF43),
            SystemHue::Green => shades(0x34C759, 0x008932, 0x30D158, 0x4AD968),
            SystemHue::Mint => shades(0x00C8B3, 0x008575, 0x00DAC3, 0x54DFCB),
            SystemHue::Teal => shades(0x00C3D0, 0x008198, 0x00D2E0, 0x3BDDEC),
            SystemHue::Cyan => shades(0x00C0E8, 0x007EAE, 0x3CD3FE, 0x6DD9FF),
            SystemHue::Blue => shades(0x0088FF, 0x1E6EF4, 0x0091FF, 0x5CB8FF),
            SystemHue::Indigo => shades(0x6155F5, 0x564ADE, 0x6D7CFF, 0xA7AAFF),
            SystemHue::Purple => shades(0xCB30E0, 0xB02FC2, 0xDB34F2, 0xEA8DFF),
            SystemHue::Pink => shades(0xFF2D55, 0xE7124D, 0xFF375F, 0xFF8AC4),
            SystemHue::Brown => shades(0xAC7F5E, 0x956D51, 0xB78A66, 0xDBA679),
        }
    }

    /// The hue in `scheme` at `contrast`.
    pub const fn value(self, scheme: Scheme, contrast: Contrast) -> Hex {
        self.shades().pick(scheme, contrast)
    }
}

#[cfg(test)]
mod tests {
    use super::SystemHue;
    use crate::appearance::{system::Contrast, theme::Scheme};
    use ds_core::word::Word;

    #[test]
    fn the_twelve_come_in_the_order_the_guidelines_list_them() {
        let slugs: Vec<&str> = SystemHue::ALL.iter().map(|hue| hue.slug()).collect();
        assert_eq!(
            slugs,
            [
                "red", "orange", "yellow", "green", "mint", "teal", "cyan", "blue", "indigo",
                "purple", "pink", "brown"
            ]
        );
    }

    #[test]
    fn blue_is_apples_blue_in_all_four_forms() {
        let got = [
            (Scheme::Light, Contrast::Normal),
            (Scheme::Light, Contrast::High),
            (Scheme::Dark, Contrast::Normal),
            (Scheme::Dark, Contrast::High),
        ]
        .map(|(scheme, contrast)| SystemHue::Blue.value(scheme, contrast).css());
        assert_eq!(
            got,
            ["#0088ff", "#1e6ef4", "#0091ff", "#5cb8ff"].map(str::to_owned)
        );
    }
}
