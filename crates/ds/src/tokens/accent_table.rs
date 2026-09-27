//! The six accents' colours, light and dark (design/03-COLOR.md sections 5 and 20).
//!
//! Every accent is generated in the settled band (B, Airy, 2026-09-27) from its hue
//! (`accent_band::hue_of`): the table holds no colour of its own. The legibility gates are the
//! band's (`accent_band::floors`) and hold at every hue, so they hold here too
//! (`tests/legibility.rs`).

use super::accent_band::{AccentPick, AccentRoles, BAND, Weight, accent_roles, hue_of};
use crate::appearance::{Accent, Scheme};

/// The roles `accent` paints in `scheme`: `--accent` (the fill), `--accent-ink`, `--accent-text`,
/// `--accent-soft` (the wash), `--accent-ring`, `--seal` (the fill).
pub fn accent_of(accent: Accent, scheme: Scheme) -> AccentRoles {
    let pick = AccentPick {
        hue: hue_of(accent),
        weight: Weight::FULL,
    };
    accent_roles(&BAND, pick, scheme)
}
