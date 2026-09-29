//! How a peek floats over the card (design/04-COMPONENTS.md section 24, `PeekMode`).
//!
//! A peek is Center or Full; a place in an app's own grid is not an overlay and stays the app's.

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
