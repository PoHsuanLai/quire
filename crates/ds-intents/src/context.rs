//! How a surface reports what it shows.

use crate::thing::ThingMark;

/// What a list, a row or an edit surface tells the companion it is showing. Components fill it
/// from their own props, so an app does nothing for context beyond naming its things.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContextModel {
    /// The thing this surface stands for, if it stands for one.
    pub thing: Option<ThingMark>,
    /// The things it lists, in the order the person sees them.
    pub things: Vec<ThingMark>,
}
