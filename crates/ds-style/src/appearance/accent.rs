//! The accent a person picks for the card (design/03-COLOR.md section 5 and open decision 6;
//! design/22-SETTINGS.md section 3.1 `appearance.accent`).
//!
//! Apple's twelve system hues and the Mac's graphite: Blue (the default), Indigo, Purple, Pink,
//! Red, Orange, Yellow, Green, Mint, Teal, Cyan, Brown, Graphite. The macOS picker's nine come
//! first in its own order; the four more are the 2025 system colours it does not offer. The
//! token values live in `tokens::accent_table`.
//! No older names are read (`postmark`, `amber`, `violet`): they fall to the default like any
//! other unknown word.

use ds_core::word::Word;
use serde::{Deserialize, Serialize};

/// The card's accent, one of thirteen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash, Default, Word)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Accent {
    /// systemBlue, the default.
    #[default]
    Blue,
    /// systemIndigo.
    Indigo,
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
    /// systemMint.
    Mint,
    /// systemTeal.
    Teal,
    /// systemCyan.
    Cyan,
    /// systemBrown.
    Brown,
    /// The Mac graphite accent, a neutral grey.
    Graphite,
}
