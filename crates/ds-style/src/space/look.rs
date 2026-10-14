//! What a Space is: up to three dots, a grain and a theme (design/21-SPACES.md section 1,
//! design/03-COLOR.md section 18). The accent is the desktop's, never a Space's.

use super::palette::{Dot, NEUTRAL_DOT};
use crate::appearance::theme::Theme;
use serde::{Deserialize, Serialize};

/// How loud the frame's grain is, 0 to 100.
///
/// The value is a percentage of the full-strength tile: opacity is `grain / 100 x .10` light
/// and `x .08` dark (design/03-COLOR.md section 8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Grain(pub u8);

impl Default for Grain {
    /// The first-run grain: none (design/03-COLOR.md section 8).
    fn default() -> Self {
        Grain(0)
    }
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
}

impl Default for SpaceLook {
    /// The neutral Space: the grey dot, the first-run grain, the system theme.
    fn default() -> Self {
        SpaceLook {
            dots: vec![NEUTRAL_DOT],
            grain: Grain::default(),
            theme: Theme::System,
        }
    }
}
