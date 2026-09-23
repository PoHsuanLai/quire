//! What a Space is: up to three dots, a grain, a theme and a card accent
//! (design/21-SPACES.md section 1, design/03-COLOR.md section 18).

use super::palette::{Dot, NEUTRAL_DOT};
use crate::appearance::Theme;
use serde::{Deserialize, Serialize};

/// How loud the frame's grain is, 0 to 100.
///
/// The value is a percentage of the full-strength tile: opacity is `grain / 100 x .20` light
/// and `x .16` dark (design/03-COLOR.md section 8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Grain(pub u8);

impl Default for Grain {
    /// The first-run grain: the Work preset's 35 (design/03-COLOR.md section 7).
    fn default() -> Self {
        Grain(35)
    }
}

/// Whether the card's accent follows the Space or stays Postmark
/// (design/22-SETTINGS.md section 3.14 `spaces.default_card_accent`).
///
/// mailo called the Space variant `Hint`; the settings doc names it `SpaceHue`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardAccent {
    /// The card keeps Postmark, whatever hue the Space is.
    #[default]
    Postmark,
    /// The card borrows the Space's hue at a quarter of a free accent's chroma.
    SpaceHue,
}

/// One Space's look: what a workspace or a mail Space paints its frame with.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SpaceLook {
    /// One to three dots, left to right across the gradient. Empty reads as the neutral dot.
    pub dots: Vec<Dot>,
    /// How loud the grain is.
    pub grain: Grain,
    /// This Space's own theme: System, Light or Dark.
    pub theme: Theme,
    /// Whether the card borrows the Space's hue.
    pub card_accent: CardAccent,
}

impl Default for SpaceLook {
    /// The neutral Space: the grey dot, the first-run grain, the system theme, Postmark.
    fn default() -> Self {
        SpaceLook {
            dots: vec![NEUTRAL_DOT],
            grain: Grain::default(),
            theme: Theme::System,
            card_accent: CardAccent::Postmark,
        }
    }
}
