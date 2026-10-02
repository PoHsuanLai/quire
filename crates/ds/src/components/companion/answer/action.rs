//! A card's buttons.

use ds_core::vocab::{Availability, EffectMark, Shortcut};
use ds_core::word::Word;

/// One card action's id: opaque, the companion's own, handed back when the person picks it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CardActionId(pub String);

/// How a card action ranks among its siblings, `data-role`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum ActionRole {
    /// The default: answers Return.
    Primary,
    /// An alternative.
    Secondary,
    /// Destroys something; never the default.
    Destructive,
}

/// A button on a card. An outbound or destructive effect is always marked, so it is never
/// drawn as the primary action without its mark.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CardAction {
    /// What the companion hears back.
    pub id: CardActionId,
    /// The button's words.
    pub label: String,
    /// Its rank.
    pub role: ActionRole,
    /// What it does to the world.
    pub effect: EffectMark,
    /// The key that does it.
    pub keys: Option<Shortcut>,
    /// Whether it takes input.
    pub availability: Availability,
}
