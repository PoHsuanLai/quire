//! The three candidate bands for the user's pick, and the hues the built-in accents name
//! (design/03-COLOR.md section 20, **proposed**). Postmark stays the default until one is picked.

use super::band::{AccentBand, ChromaSpan, Hue, InkRule, Milli, SchemeBand};
use crate::appearance::Accent;
use crate::tokens::Alpha;

/// A candidate band.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Candidate {
    /// A: the Mac's system-blue family. Vivid mid-light fills (chroma up to .19) with white ink,
    /// light washes.
    System,
    /// B: airy. Pastel fills (L .80, chroma up to .10) with a deep ink of the hue, the text
    /// accent a dusty mid-tone, stronger washes to show on white.
    Airy,
    /// C: calm, between the two. White-ink fills at chroma .14 in light, pastel deep-ink fills
    /// in dark (as Postmark's dark quad is).
    Calm,
}

impl Candidate {
    /// Every candidate, in the sheet's order.
    pub const ALL: [Candidate; 3] = [Candidate::System, Candidate::Airy, Candidate::Calm];

    /// The sheet's file slug.
    pub fn slug(self) -> &'static str {
        match self {
            Candidate::System => "a-system",
            Candidate::Airy => "b-airy",
            Candidate::Calm => "c-calm",
        }
    }

    /// The sheet's column title.
    pub fn label(self) -> &'static str {
        match self {
            Candidate::System => "A · System",
            Candidate::Airy => "B · Airy",
            Candidate::Calm => "C · Calm",
        }
    }

    /// The band.
    pub fn band(self) -> AccentBand {
        match self {
            Candidate::System => SYSTEM,
            Candidate::Airy => AIRY,
            Candidate::Calm => CALM,
        }
    }
}

/// The hue a built-in accent generates from: each Candy family's own hue (`#E8483C` 28,
/// `#F0A81E` 76, `#28B24A` 147, `#2B7CFF` 260, `#8B5CF0` 294), Postmark's `#23508F` 257.
///
/// Inside one band Postmark and Blue come out nearly the same colour; which of them survives is
/// part of the pick.
pub fn hue_of(accent: Accent) -> Hue {
    Hue(match accent {
        Accent::Postmark => 257,
        Accent::Red => 28,
        Accent::Amber => 76,
        Accent::Green => 147,
        Accent::Blue => 260,
        Accent::Violet => 294,
    })
}

const fn span(quiet: u16, full: u16) -> ChromaSpan {
    ChromaSpan {
        quiet: Milli(quiet),
        full: Milli(full),
    }
}

const SYSTEM: AccentBand = AccentBand {
    light: SchemeBand {
        fill: Milli(620),
        chroma: span(40, 190),
        ink: InkRule::White,
        text: Milli(580),
        wash: Alpha(160),
        ring: Alpha(500),
    },
    dark: SchemeBand {
        fill: Milli(620),
        chroma: span(40, 170),
        ink: InkRule::White,
        text: Milli(720),
        wash: Alpha(260),
        ring: Alpha(500),
    },
};

const AIRY: AccentBand = AccentBand {
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

const CALM: AccentBand = AccentBand {
    light: SchemeBand {
        fill: Milli(640),
        chroma: span(40, 140),
        ink: InkRule::White,
        text: Milli(560),
        wash: Alpha(180),
        ring: Alpha(500),
    },
    dark: SchemeBand {
        fill: Milli(740),
        chroma: span(40, 120),
        ink: InkRule::Deep,
        text: Milli(740),
        wash: Alpha(220),
        ring: Alpha(500),
    },
};
