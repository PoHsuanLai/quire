//! Menus, for mail: a people menu whose highlight a field beside it drives, and a menu's items
//! inline in a sender card.

use dioxus::prelude::*;

/// Both mail specimens.
#[component]
pub fn FieldAndCard() -> Element {
    rsx! {
        crate::pages::overlays::palette_and_menu::FieldMenu {}
        crate::pages::overlays::hover_card_hooks::InlineActions {}
    }
}
