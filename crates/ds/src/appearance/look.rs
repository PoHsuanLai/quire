//! The look axis and Candy's warmth (design/07-LOOKS.md section 2; design/22-SETTINGS.md
//! section 3.1 `appearance.look` and `appearance.warmth`).

use serde::{Deserialize, Serialize};

/// A whole visual language. Post (as the Spaces prototype draws it) is the desktop default;
/// the others are optional themes kept in the token table (design/07-LOOKS.md section 11).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash, Default)]
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

impl Look {
    /// Every look, in the order a picker offers them.
    pub const ALL: [Look; 4] = [Look::Post, Look::Riso, Look::Tide, Look::Candy];

    /// The stored and attribute word.
    pub fn slug(self) -> &'static str {
        match self {
            Look::Post => "post",
            Look::Riso => "riso",
            Look::Tide => "tide",
            Look::Candy => "candy",
        }
    }
}

/// How warm Candy's neutrals run. Applies only when the look is Candy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash, Default)]
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

impl Warmth {
    /// Every step, coolest first.
    pub const ALL: [Warmth; 4] = [Warmth::Cool, Warmth::Neutral, Warmth::Warm, Warmth::Paper];

    /// The stored and attribute word.
    pub fn slug(self) -> &'static str {
        match self {
            Warmth::Cool => "cool",
            Warmth::Neutral => "neutral",
            Warmth::Warm => "warm",
            Warmth::Paper => "paper",
        }
    }
}
