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

impl CountPlace {
    /// The `data-place` word.
    fn slug(self) -> &'static str {
        match self {
            CountPlace::Item => "item",
            CountPlace::Tile => "tile",
        }
    }
}

/// The text a count shows: nothing at zero, so the layout does not shift (`S:1247`).
fn text(value: u32) -> String {
    match value {
        0 => String::new(),
        n => n.to_string(),
    }
}

/// A count. `pulse` is the consumer's `use_pulse(Anim::Bump)`: it fires the bump when the
/// value changes, and leaves it at rest across a Space switch (O-8).
#[component]
pub fn Count(value: u32, #[props(default)] place: CountPlace, pulse: PulseKey) -> Element {
    let (class, alias) = match pulse.attrs() {
        Some((anim, alias)) => (format!("ds-count {anim}"), Some(alias)),
        None => ("ds-count".to_string(), None),
    };
    rsx! {
        span {
            class,
            "data-place": place.slug(),
            "data-pulse": alias,
            {text(value)}
        }
    }
}
