//! RowAction: a button at the end of a menu or palette row that acts on that row without
//! picking it (mailo gaps 2: the × that removes a recent search). It is an `IconButton`, and
//! its click, press and pointer moves stop inside it, so the row neither runs, closes its menu
//! nor takes the selection while the pointer is on the button.

use crate::components::icon_button::{IconButton, IconButtonVariant};
use crate::components::press::Press;
use crate::focus::click::kept_click;
use crate::icon::Icon;
use dioxus::prelude::*;

/// A row's trailing action.
#[derive(Debug, Clone, PartialEq)]
pub struct RowAction {
    /// Its glyph (`Icon::X` for a remove).
    pub icon: Icon,
    /// Its `aria-label`, and its `title`: "Remove from recent".
    pub label: String,
    /// Pressed: the row is not picked and the menu stays open.
    pub on_press: EventHandler<Press>,
}

/// `action` at the end of a row, fenced so nothing it hears reaches the row.
pub(crate) fn trailing(action: &RowAction) -> Element {
    let RowAction {
        icon,
        label,
        on_press,
    } = action.clone();
    rsx! {
        span {
            class: "ds-menu-action",
            // A click here is the action's, never the row's pick.
            onclick: move |event| {
                event.stop_propagation();
                kept_click(&event);
            },
            // The row keeps the focus where it was and does not start a press-drag-release.
            onmousedown: move |event| {
                event.prevent_default();
                event.stop_propagation();
            },
            onmouseup: move |event| event.stop_propagation(),
            // Pointing at the action does not move the selection onto its row.
            onmousemove: move |event| event.stop_propagation(),
            IconButton {
                variant: IconButtonVariant::Strip,
                icon,
                label: label.clone(),
                tooltip: label,
                onclick: on_press,
            }
        }
    }
}
