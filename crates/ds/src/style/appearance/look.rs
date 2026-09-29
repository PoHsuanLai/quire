//! The look axis and Candy's warmth (design/07-LOOKS.md section 2; design/22-SETTINGS.md
//! section 3.1 `appearance.look` and `appearance.warmth`).

use crate::core::word::Word;
use serde::{Deserialize, Serialize};

/// A whole visual language. Post (as the Spaces prototype draws it) is the desktop default;
/// the others are optional themes kept in the token table (design/07-LOOKS.md section 11).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash, Default, Word)]
#[serde(rename_all = "snake_case")]
pub enum Look {
    /// Paper and ink, Postmark blue.
    #[default]
    Post,
    /// Print-shop overprint.
    Riso,
    /// Slow, soft, no spring.
    Tide,
    /// The candy shelf.
    Candy,
}

/// How warm Candy's neutrals run. Applies only when the look is Candy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash, Default, Word)]
#[serde(rename_all = "snake_case")]
pub enum Warmth {
    /// Candy's own base.
    Cool,
    /// The settings default.
    #[default]
    Neutral,
    /// Warmer soft and deep members.
    Warm,
    /// The warmest step.
    Paper,
}
