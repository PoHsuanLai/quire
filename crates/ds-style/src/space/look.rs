//! What a Space is: up to three dots, a theme and a card accent
//! (design/21-SPACES.md section 1, design/03-COLOR.md section 18).

use super::palette::{Dot, NEUTRAL_DOT};
use crate::appearance::theme::Theme;
use serde::{Deserialize, Serialize};

/// Whether the card's accent follows the Space or stays the accent the person picked
/// (design/22-SETTINGS.md section 3.14 `spaces.default_card_accent`).
///
/// mailo called the Space variant `Hint`; the settings doc names it `SpaceHue`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardAccent {
    /// The card keeps the chosen accent, whatever hue the Space is.
    #[default]
    Chosen,
    /// The card borrows the Space's hue at a quarter of a free accent's chroma.
    SpaceHue,
}

/// One Space's look: what a workspace or a mail Space paints its frame with.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SpaceLook {
    /// One to three dots, left to right across the gradient. Empty reads as the neutral dot.
    pub dots: Vec<Dot>,
    /// This Space's own theme: System, Light or Dark.
    pub theme: Theme,
    /// Whether the card borrows the Space's hue.
    pub card_accent: CardAccent,
}

impl Default for SpaceLook {
    /// The neutral Space: the grey dot, the system theme, the chosen accent.
    fn default() -> Self {
        SpaceLook {
            dots: vec![NEUTRAL_DOT],
            theme: Theme::System,
            card_accent: CardAccent::Chosen,
        }
    }
}
