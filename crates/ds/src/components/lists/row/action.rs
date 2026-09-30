//! RowAction: a button at the end of a row that acts on that row without picking it (the ×
//! that removes a recent search, the ⋯ that opens the row's menu). It is an `IconButton`, and its
//! click, press and pointer moves stop inside it, so the row neither runs, closes its palette
//! nor takes the selection while the pointer is on the button.
//!
//! A menu hangs from it through `common.mounted`: the caller keeps the button's element and
//! anchors the menu to it (`Anchor::Mounted`). A row whose extra commands are one menu puts a
//! `PopUpButton` of `PopUpKind::Overflow` in its accessory slot instead.

use crate::components::content::icon_source::IconSource;
use crate::components::controls::button::Button;
use crate::components::controls::button_model::{Bezel, ImagePosition};
use crate::focus::click::kept_click;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::press::Press;
use ds_style::icon::Icon;
use ds_style::tokens::control_size::ControlSize;

/// A row's trailing action.
#[derive(Debug, Clone, PartialEq)]
pub struct RowAction {
    /// Its glyph (`Icon::X` for a remove).
    pub icon: Icon,
    /// Its `aria-label`, and its `title`: "Remove from recent".
    pub label: String,
    /// Pressed: the row is not picked and the palette stays open.
    pub on_press: EventHandler<Press>,
    /// The consumer's `id`, `data-*`, classes and `mounted` handle for the button, so a menu the
    /// caller builds can hang from it.
    pub common: Common,
}

impl RowAction {
    /// An action of `icon` named `label`, running `on_press`.
    pub fn new(icon: Icon, label: impl Into<String>, on_press: EventHandler<Press>) -> Self {
        RowAction {
            icon,
            label: label.into(),
            on_press,
            common: Common::default(),
        }
    }

    /// The same action with `common` on its button.
    pub fn with_common(self, common: Common) -> Self {
        RowAction { common, ..self }
    }
}

/// `action` at the end of a row, fenced so nothing it hears reaches the row.
pub(crate) fn trailing(action: &RowAction) -> Element {
    let RowAction {
        icon,
        label,
        on_press,
        common,
    } = action.clone();
    rsx! {
        span {
            class: "ds-row-action",
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
            onpointermove: move |event| event.stop_propagation(),
            Button {
                bezel: Bezel::Toolbar,
                image: ImagePosition::Only,
                icon: Some(IconSource::Glyph(icon)),
                label: label.clone(),
                title: Some(label),
                size: ControlSize::Small,
                onclick: on_press,
                common,
            }
        }
    }
}
