//! The accents' colours, light and dark, at ordinary and increased contrast (design/03-COLOR.md
//! sections 5 and 20).
//!
//! Every accent is one of Apple's 2025 system hues (`system_hue`) that the macOS picker offers, except graphite, the Mac's
//! neutral grey. The fill is the hue's own value, so a selected tab, a button and the focus
//! ring's colour keep Apple's blue; the text accent (`--accent-text`: a link, the caret, a
//! chip's words) leads from the hue's increased-contrast value and steps further only where a
//! ground in the band (`accent_band::floors`) would not read it. At increased contrast the fill
//! is the increased value too. The legibility gates hold at every hue (`tests/legibility.rs`).

use crate::appearance::{accent::Accent, system::Contrast, theme::Scheme};
use crate::tokens::accent_band::{derive::system_roles, picked::BAND, roles::AccentRoles};
use crate::tokens::hex::Hex;
use crate::tokens::system_hue::{HueShades, SystemHue};

/// The Mac's graphite accent: systemGray, in its four forms.
const GRAPHITE: HueShades = HueShades {
    light: Hex([0x8E, 0x8E, 0x93]),
    light_increased: Hex([0x6C, 0x6C, 0x70]),
    dark: Hex([0x98, 0x98, 0x9D]),
    dark_increased: Hex([0xAE, 0xAE, 0xB2]),
};

/// The roles `accent` paints in `scheme` at ordinary contrast: `--accent` (the fill),
/// `--accent-ink`, `--accent-text`, `--accent-soft` (the wash), `--accent-ring`, `--seal` (the fill).
pub fn accent_of(accent: Accent, scheme: Scheme) -> AccentRoles {
    accent_in(accent, scheme, Contrast::Normal)
}

/// The roles `accent` paints in `scheme` at `contrast`.
pub fn accent_in(accent: Accent, scheme: Scheme, contrast: Contrast) -> AccentRoles {
    let shades = shades_of(accent);
    let lead = shades.pick(scheme, Contrast::High);
    system_roles(&BAND, scheme, shades.pick(scheme, contrast), lead)
}

/// The four values an accent stands for.
fn shades_of(accent: Accent) -> HueShades {
    let hue = match accent {
        Accent::Blue => SystemHue::Blue,
        Accent::Purple => SystemHue::Purple,
        Accent::Pink => SystemHue::Pink,
        Accent::Red => SystemHue::Red,
        Accent::Orange => SystemHue::Orange,
        Accent::Yellow => SystemHue::Yellow,
        Accent::Green => SystemHue::Green,
        Accent::Graphite => return GRAPHITE,
    };
    hue.shades()
}
