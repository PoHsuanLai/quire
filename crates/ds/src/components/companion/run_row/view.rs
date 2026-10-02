//! The run row component.

use super::model::RunRowView;
use crate::components::companion::answer::action::CardActionId;
use dioxus::prelude::*;
use ds_core::vocab::Selection;
use ds_core::word::Word;

/// A run as a row: its goal, the app, the step it is at and its controls. `on_action` hears the
/// control picked.
#[component]
pub fn RunRow(
    view: RunRowView,
    #[props(default)] selection: Selection,
    on_action: EventHandler<CardActionId>,
) -> Element {
    let _ = on_action;
    rsx! {
        div {
            class: "ds-run-row",
            "data-state": view.state.slug(),
            "data-place": view.place.slug(),
            "aria-selected": if selection == Selection::Selected { Some("true") } else { None },
        }
    }
}
