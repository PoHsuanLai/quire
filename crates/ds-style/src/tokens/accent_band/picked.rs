//! The band the user picked (B, Airy; design/03-COLOR.md section 20, settled 2026-09-27): the
//! bounds a Space's lent accent is generated in, and the chroma and text/wash/ring bounds the
//! built-in accents' Mac system fills are measured in (`derive::system_roles`).
//!
//! Airy: pastel fills (OKLCH L .80 light, .78 dark, chroma up to .10) carrying a deep ink of
//! their own hue, a dusty mid-tone text accent, translucent washes. The two bands the user did
//! not pick (A System, C Calm) are recorded with their numbers in section 20.

use super::band::{AccentBand, ChromaSpan, InkRule, Milli, SchemeBand};
use crate::tokens::hex::Alpha;

/// The band every accent is generated in.
pub const BAND: AccentBand = AccentBand {
    light: SchemeBand {
        fill: Milli(800),
        chroma: span(30, 100),
        ink: InkRule::Deep,
        text: Milli(540),
        wash: Alpha(180),
        ring: Alpha(500),
    },
    dark: SchemeBand {
        fill: Milli(780),
        chroma: span(30, 100),
        ink: InkRule::Deep,
        text: Milli(760),
        wash: Alpha(200),
        ring: Alpha(500),
    },
};

const fn span(quiet: u16, full: u16) -> ChromaSpan {
    ChromaSpan {
        quiet: Milli(quiet),
        full: Milli(full),
    }
}
