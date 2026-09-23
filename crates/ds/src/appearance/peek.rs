//! How a peek floats over the card (design/04-COMPONENTS.md section 24, `PeekMode`).
//!
//! Moved from mailo's `view::Peek`. mailo's `Side` variant is the reader's place in mailo's own
//! grid, not an overlay, so it stays in mailo; quire's peek is Center or Full.

use serde::{Deserialize, Serialize};

/// Where an open peek sits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum PeekMode {
    /// A panel over the middle of the card.
    #[default]
    Center,
    /// The whole card.
    Full,
}

impl PeekMode {
    /// `center` or `full`, written as `data-mode`.
    pub fn slug(self) -> &'static str {
        match self {
            PeekMode::Center => "center",
            PeekMode::Full => "full",
        }
    }

    /// What the peek button names itself: "Centre peek", "Full page".
    pub fn label(self) -> &'static str {
        match self {
            PeekMode::Center => "Centre peek",
            PeekMode::Full => "Full page",
        }
    }
}
