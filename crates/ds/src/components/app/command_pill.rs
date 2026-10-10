//! CommandPill: "Search or run a command", on the frame (design/04-COMPONENTS.md section 8).

use crate::keys::use_keys;
use dioxus::prelude::*;
use ds_core::vocab::Shortcut;
use ds_style::icon::Icon;
use ds_style::icon::render::{Glyph, IconSize};

/// The full-width pill that opens the command palette.
#[component]
pub fn CommandPill(label: String, shortcut: Shortcut, onclick: EventHandler<()>) -> Element {
    let keys = use_keys().text_of(&shortcut);
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
