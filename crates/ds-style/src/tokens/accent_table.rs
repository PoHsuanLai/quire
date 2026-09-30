//! The eight accents' colours, light and dark (design/03-COLOR.md sections 5 and 20).
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

/// The macOS system colour an accent stands for (Sonoma and Sequoia, default appearance); the
/// Mac's graphite accent is its neutral grey (systemGray).
fn system_fill(accent: Accent, scheme: Scheme) -> Hex {
    let (light, dark) = match accent {
        Accent::Blue => ([0x00, 0x7A, 0xFF], [0x0A, 0x84, 0xFF]),
        Accent::Purple => ([0xAF, 0x52, 0xDE], [0xBF, 0x5A, 0xF2]),
        Accent::Pink => ([0xFF, 0x2D, 0x55], [0xFF, 0x37, 0x5F]),
        Accent::Red => ([0xFF, 0x3B, 0x30], [0xFF, 0x45, 0x3A]),
        Accent::Orange => ([0xFF, 0x95, 0x00], [0xFF, 0x9F, 0x0A]),
        Accent::Yellow => ([0xFF, 0xCC, 0x00], [0xFF, 0xD6, 0x0A]),
        Accent::Green => ([0x34, 0xC7, 0x59], [0x30, 0xD1, 0x58]),
        Accent::Graphite => ([0x8E, 0x8E, 0x93], [0x98, 0x98, 0x9D]),
    };
    Hex(match scheme {
        Scheme::Light => light,
        Scheme::Dark => dark,
    })
}
