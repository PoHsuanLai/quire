//! The band the user picked (B, Airy; design/03-COLOR.md section 20, settled 2026-09-27) and the
//! hues the built-in accents name.
//!
//! Airy: pastel fills (OKLCH L .80 light, .78 dark, chroma up to .10) carrying a deep ink of
//! their own hue, a dusty mid-tone text accent, translucent washes. The two bands the user did
//! not pick (A System, C Calm) are recorded with their numbers in section 20.

use super::band::{AccentBand, ChromaSpan, Hue, InkRule, Milli, SchemeBand};
use crate::appearance::accent::Accent;
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

/// The hue a built-in accent generates from: each Candy family's own hue (`#E8483C` 28,
/// `#F0A81E` 76, `#28B24A` 147, `#8B5CF0` 294), Postmark's `#23508F` 257.
///
/// Blue is 215, a sky blue, not Candy's 260: at 260 it came out of the band 0.005 (OKLab) from
/// Postmark, the same swatch twice in the picker. 215 stands 0.07 from Postmark and 0.11 from
/// Green, at least the picker's narrowest existing gap (Violet to Postmark, 0.064);
/// `tests::every_built_in_swatch_is_distinct` holds it there.
pub fn hue_of(accent: Accent) -> Hue {
    Hue(match accent {
        Accent::Postmark => 257,
        Accent::Red => 28,
        Accent::Amber => 76,
        Accent::Green => 147,
        Accent::Blue => 215,
        Accent::Violet => 294,
    })
}

const fn span(quiet: u16, full: u16) -> ChromaSpan {
    ChromaSpan {
        quiet: Milli(quiet),
        full: Milli(full),
    }
}
