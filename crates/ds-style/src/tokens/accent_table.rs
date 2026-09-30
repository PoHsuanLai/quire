//! The six accents' colours, light and dark (design/03-COLOR.md sections 5 and 20).
//!
//! Every accent is generated in the settled band (B, Airy, 2026-09-27) from its hue
//! (`accent_band::hue_of`): the table holds no colour of its own. The legibility gates are the
//! band's (`accent_band::floors`) and hold at every hue, so they hold here too
//! (`tests/legibility.rs`).

use crate::appearance::{accent::Accent, theme::Scheme};
use crate::tokens::accent_band::{derive::system_roles, picked::BAND, roles::AccentRoles};
use crate::tokens::hex::Hex;

/// The roles `accent` paints in `scheme`: `--accent` (the fill), `--accent-ink`, `--accent-text`,
/// `--accent-soft` (the wash), `--accent-ring`, `--seal` (the fill).
pub fn accent_of(accent: Accent, scheme: Scheme) -> AccentRoles {
    system_roles(&BAND, scheme, system_fill(accent, scheme))
}

/// The macOS system colour an accent stands for (Sonoma and Sequoia, default appearance):
/// Postmark, the default, and so the Mac Look's accent, is systemBlue; Red is systemRed, Amber
/// systemOrange, Green systemGreen, Violet systemPurple and Blue systemTeal (systemBlue is
/// Postmark's, and a picker of two identical swatches is no picker).
fn system_fill(accent: Accent, scheme: Scheme) -> Hex {
    let (light, dark) = match accent {
        Accent::Postmark => ([0x00, 0x7A, 0xFF], [0x0A, 0x84, 0xFF]),
        Accent::Red => ([0xFF, 0x3B, 0x30], [0xFF, 0x45, 0x3A]),
        Accent::Amber => ([0xFF, 0x95, 0x00], [0xFF, 0x9F, 0x0A]),
        Accent::Green => ([0x34, 0xC7, 0x59], [0x30, 0xD1, 0x58]),
        Accent::Blue => ([0x30, 0xB0, 0xC7], [0x40, 0xC8, 0xE0]),
        Accent::Violet => ([0xAF, 0x52, 0xDE], [0xBF, 0x5A, 0xF2]),
    };
    Hex(match scheme {
        Scheme::Light => light,
        Scheme::Dark => dark,
    })
}
