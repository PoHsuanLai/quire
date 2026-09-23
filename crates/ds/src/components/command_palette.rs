//! CommandPalette: "the same menu, just bigger and centred" (design/04-COMPONENTS.md section 25).

use crate::components::menu_entry::MenuEntry;
use dioxus::prelude::*;

/// Search and commands over a scrim.
#[component]
pub fn CommandPalette<T: Clone + PartialEq + 'static>(
    label: String,
    placeholder: String,
    query: String,
    tokens: Vec<String>,
    groups: Vec<(String, Vec<MenuEntry<T>>)>,
    empty: String,
    oninput: EventHandler<String>,
    onpick: EventHandler<T>,
    onclose: EventHandler<()>,
) -> Element {
    todo!()
}
