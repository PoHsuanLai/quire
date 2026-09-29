//! Count: an unread or item count, empty at zero (design/04-COMPONENTS.md section 14).

use crate::core::word::Word;
use dioxus::prelude::*;

/// Where a count sits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum CountPlace {
    /// Trailing a sidebar item.
    #[default]
    Item,
    /// In an account tile's corner.
    Tile,
}

/// The text a count shows: nothing at zero, so the layout does not shift (`S:1247`).
fn text(value: u32) -> String {
    match value {
        0 => String::new(),
        n => n.to_string(),
    }
}

/// A count: an unread or item count, empty at zero, drawn as it is.
#[component]
pub fn Count(value: u32, #[props(default)] place: CountPlace) -> Element {
    rsx! {
        span { class: "ds-count", "data-place": place.slug(), {text(value)} }
    }
}
