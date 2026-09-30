//! How a peek floats over the card (design/04-COMPONENTS.md section 24, `PeekMode`).
//!
//! A peek is Center or Full; a place in an app's own grid is not an overlay and stays the app's.

use ds_core::word::Word;
use serde::{Deserialize, Serialize};

/// Where an open peek sits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash, Default, Word)]
#[serde(rename_all = "snake_case")]
pub enum PeekMode {
    /// A panel over the middle of the card.
    #[default]
    #[word(label = "Centre peek")]
    Center,
    /// The whole card.
    #[word(label = "Full page")]
    Full,
}
