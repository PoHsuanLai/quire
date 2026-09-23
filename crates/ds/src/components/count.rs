//! Count: an unread or item count, empty at zero, bumping when it changes
//! (design/04-COMPONENTS.md section 14).

use crate::components::vocab::PulseKey;
use dioxus::prelude::*;

/// Where a count sits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CountPlace {
    /// Trailing a sidebar item.
    #[default]
    Item,
    /// In an account tile's corner.
    Tile,
}

/// A count.
#[component]
pub fn Count(value: u32, #[props(default)] place: CountPlace, pulse: PulseKey) -> Element {
    todo!()
}
