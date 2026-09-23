//! CommandPill: "Search or run a command", on the frame (design/04-COMPONENTS.md section 8).

use crate::components::vocab::Shortcut;
use crate::icon::Icon;
use crate::icon::render::{Glyph, IconSize};
use dioxus::prelude::*;

/// The full-width pill that opens the command palette.
#[component]
pub fn CommandPill(label: String, shortcut: Shortcut, onclick: EventHandler<()>) -> Element {
    let keys = shortcut.glyphs();
    rsx! {
        button {
            r#type: "button",
            class: "ds-command-pill",
            onclick: move |_| onclick.call(()),
            Glyph { icon: Icon::Search, size: IconSize::Base }
            span { class: "ds-command-pill-label ds-truncate", "{label}" }
            span { class: "ds-command-pill-keys", "{keys}" }
        }
    }
}
