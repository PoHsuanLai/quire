//! The accent a person picks for the card (design/03-COLOR.md section 5 and open decision 6;
//! design/22-SETTINGS.md section 3.1 `appearance.accent`).
//!
//! Six accents: Postmark, the design's own blue, and one per Candy hue (design/03-COLOR.md
//! section 15). The five hue names are this freeze's reading of "Accent(6)" and are recorded
//! in FINDINGS.md; the token values live in `tokens::accent_table`.

use serde::{Deserialize, Serialize};

/// The card's accent, one of six.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum Accent {
    /// Postmark blue, the design as drawn.
    #[default]
    Postmark,
    /// The Candy red family.
    Red,
    /// The Candy amber family.
    Amber,
    /// The Candy green family.
    Green,
    /// The Candy blue family.
    Blue,
    /// The Candy violet family.
    Violet,
}

impl Accent {
    /// Every accent, in the order the picker offers them.
    pub const ALL: [Accent; 6] = [
        Accent::Postmark,
        Accent::Red,
        Accent::Amber,
        Accent::Green,
        Accent::Blue,
        Accent::Violet,
    ];

    /// The `data-accent` value on a `.ds` root.
    pub fn slug(self) -> &'static str {
        match self {
            Accent::Postmark => "postmark",
            Accent::Red => "red",
            Accent::Amber => "amber",
            Accent::Green => "green",
            Accent::Blue => "blue",
            Accent::Violet => "violet",
        }
    }

    /// What a picker calls it.
    pub fn label(self) -> &'static str {
        match self {
            Accent::Postmark => "Postmark",
            Accent::Red => "Red",
            Accent::Amber => "Amber",
            Accent::Green => "Green",
            Accent::Blue => "Blue",
            Accent::Violet => "Violet",
        }
    }

    /// The accent a stored word names, or [`None`] for a word that is not one.
    pub fn parse(word: &str) -> Option<Accent> {
        Self::ALL.into_iter().find(|accent| accent.slug() == word)
    }
}
