//! The accent a person picks for the card (design/03-COLOR.md section 5 and open decision 6;
//! design/22-SETTINGS.md section 3.1 `appearance.accent`).
//!
//! macOS's own list of eight, in the order its picker shows them: Blue (the default), Purple,
//! Pink, Red, Orange, Yellow, Green, Graphite. The token values live in `tokens::accent_table`.
//! No older names are read (`postmark`, `amber`, `violet`): they fall to the default like any
//! other unknown word.

use ds_core::word::Word;
use serde::{Deserialize, Serialize};

/// The card's accent, one of eight.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash, Default, Word)]
#[serde(rename_all = "snake_case")]
pub enum Accent {
    /// systemBlue, the default.
    #[default]
    Blue,
    /// systemPurple.
    Purple,
    /// systemPink.
    Pink,
    /// systemRed.
    Red,
    /// systemOrange.
    Orange,
    /// systemYellow.
    Yellow,
    /// systemGreen.
    Green,
    /// The Mac graphite accent, a neutral grey.
    Graphite,
}
