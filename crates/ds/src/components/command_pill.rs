//! CommandPill: "Search or run a command", on the frame (design/04-COMPONENTS.md section 8).

use crate::components::vocab::Shortcut;
use dioxus::prelude::*;

/// The full-width pill that opens the command palette.
#[component]
pub fn CommandPill(label: String, shortcut: Shortcut, onclick: EventHandler<()>) -> Element {
    todo!()
}
