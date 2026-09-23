//! The six accents, four properties each, light and dark (the plan's "6 hues x 4 props";
//! design/03-COLOR.md section 5 and open decision 6).
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

use super::hex::Hex;
use crate::appearance::{Accent, Scheme};

/// The four properties one accent sets: `--accent`, `--accent-ink`, `--accent-soft`, `--seal`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AccentQuad {
    /// `--accent`.
    pub accent: Hex,
    /// `--accent-ink`: text on the accent.
    pub ink: Hex,
    /// `--accent-soft`: the accent's tint.
    pub soft: Hex,
    /// `--seal`.
    pub seal: Hex,
}

/// The quad `accent` paints in `scheme`.
pub fn quad(accent: Accent, scheme: Scheme) -> AccentQuad {
    todo!()
}
