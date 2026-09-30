//! Menus, for mail: a people menu whose highlight a field beside it drives, and a menu's items
//! inline in a sender card.

use dioxus::prelude::*;

/// Both mail specimens.
#[component]
pub fn FieldAndCard() -> Element {
    rsx! {
        super::overlays_mailo::FieldMenu {}
        super::overlays_mailo4::InlineActions {}
    }
}
