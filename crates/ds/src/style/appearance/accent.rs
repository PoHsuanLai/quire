//! The accent a person picks for the card (design/03-COLOR.md section 5 and open decision 6;
//! design/22-SETTINGS.md section 3.1 `appearance.accent`).
//!
//! Six accents: Postmark, the design's own blue, and one per Candy hue (design/03-COLOR.md
//! section 15). The five hue names are this freeze's reading of "Accent(6)" and are recorded
//! in FINDINGS.md; the token values live in `tokens::accent_table`.

use crate::core::word::Word;
use serde::{Deserialize, Serialize};

/// The card's accent, one of six.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash, Default, Word)]
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
