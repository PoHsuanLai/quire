//! A persona's description (design/24-PERSONA.md section 3): what it is, and one small enum per
//! part. It is user data, saved by the Settings app, so it is serde and every part has a default.

use serde::{Deserialize, Serialize};

/// What the character is: a person, one of three round animals, or a blob.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Creature {
    /// A person: skin, hair, no ears.
    Person,
    /// A bear: round ears and a muzzle.
    Bear,
    /// A cat: pointed ears and whiskers.
    Cat,
    /// A rabbit: two tall ears.
    Bunny,
    /// A blob: no ears, a sprout on top.
    Blob,
}

/// The head's outline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeadShape {
    /// Nearly a circle.
    Round,
    /// A soft rounded square.
    Soft,
    /// Taller than wide.
    Tall,
    /// Wider than tall.
    Wide,
}

/// Skin or fur: five skin tones and the icon palette's eight hues (design/08 section 2.10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tone {
    /// The lightest skin.
    Peach,
    /// Light skin.
    Sand,
    /// Medium skin.
    Honey,
    /// Brown skin.
    Umber,
    /// The deepest skin.
    Cocoa,
    /// Fur in clay, hue 40.
    Clay,
    /// Fur in ochre, hue 85.
    Ochre,
    /// Fur in sage, hue 130.
    Sage,
    /// Fur in jade, hue 175.
    Jade,
    /// Fur in teal, hue 220.
    Teal,
    /// Fur in slate, hue 265.
    Slate,
    /// Fur in plum, hue 310.
    Plum,
    /// Fur in rose, hue 355.
    Rose,
}

/// The eyes when open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Eyes {
    /// Two round dots.
    Dot,
    /// Two upright ovals.
    Oval,
    /// Two larger dots with a light fleck.
    Shine,
    /// Two dots set further apart.
    Wide,
}

/// The brows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Brows {
    /// No brows.
    Hidden,
    /// Two short arcs.
    Soft,
    /// Two short straight strokes.
    Straight,
}

/// The mouth at rest (Idle); the other moods draw their own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mouth {
    /// A curved smile.
    Smile,
    /// A small `w`.
    Cat,
    /// A small closed smile.
    Small,
    /// An open grin.
    Grin,
}

/// What sits on top of the head: hair for a person, a tuft or the same shapes on an animal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Top {
    /// Nothing.
    Bare,
    /// A single curl.
    Tuft,
    /// A short cap.
    Crop,
    /// A cap with scalloped bangs.
    Fringe,
    /// A bob with a swept fringe.
    Bob,
    /// A cap and a bun.
    Bun,
    /// Curls round the crown.
    Curly,
}

/// The hair's colour (and a tuft's or a sprout's).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HairTone {
    /// Near black.
    Ink,
    /// Dark brown.
    Cocoa,
    /// Red brown.
    Auburn,
    /// Golden.
    Honey,
    /// Pale grey.
    Silver,
    /// Bright rose.
    Rose,
    /// Bright teal.
    Teal,
    /// Bright plum.
    Plum,
}

/// Cheeks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cheeks {
    /// The face's own tone.
    Plain,
    /// A blush on each cheek.
    Blush,
}

/// One small extra.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Accessory {
    /// Nothing.
    Plain,
    /// Round glasses.
    Glasses,
    /// Freckles on the cheeks.
    Freckles,
    /// A bow at the side of the head.
    Bow,
}

/// The disc behind the character: one of the icon palette's eight hues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Backdrop {
    /// Hue 40.
    Clay,
    /// Hue 85.
    Ochre,
    /// Hue 130.
    Sage,
    /// Hue 175.
    Jade,
    /// Hue 220.
    Teal,
    /// Hue 265.
    Slate,
    /// Hue 310.
    Plum,
    /// Hue 355.
    Rose,
}

/// The seed a persona was drawn from; it also sets the rhythm of its blinks, so two personas
/// side by side do not blink in step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PersonaSeed(pub u64);

/// A persona: the user's own character, drawn from simple shapes (design/24-PERSONA.md).
///
/// Every field defaults, so a spec saved by an older Settings (or a partial one) still loads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(default)]
pub struct PersonaSpec {
    /// What it is.
    pub creature: Creature,
    /// The head's outline.
    pub head: HeadShape,
    /// Skin or fur.
    pub tone: Tone,
    /// The eyes when open.
    pub eyes: Eyes,
    /// The brows.
    pub brows: Brows,
    /// The resting mouth.
    pub mouth: Mouth,
    /// Hair, a tuft, or nothing.
    pub top: Top,
    /// The hair's colour.
    pub hair: HairTone,
    /// Cheeks.
    pub cheeks: Cheeks,
    /// One small extra.
    pub accessory: Accessory,
    /// The disc behind it.
    pub backdrop: Backdrop,
    /// The blink rhythm.
    pub seed: PersonaSeed,
}

impl Default for PersonaSpec {
    /// An ochre blob with a sprout on a teal disc: no skin tone or hair assumed for a person
    /// who has not made their own yet.
    fn default() -> Self {
        PersonaSpec {
            creature: Creature::Blob,
            head: HeadShape::Round,
            tone: Tone::Ochre,
            eyes: Eyes::Dot,
            brows: Brows::Hidden,
            mouth: Mouth::Smile,
            top: Top::Tuft,
            hair: HairTone::Teal,
            cheeks: Cheeks::Blush,
            accessory: Accessory::Plain,
            backdrop: Backdrop::Teal,
            seed: PersonaSeed(0),
        }
    }
}
