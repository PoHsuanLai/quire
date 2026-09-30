//! SectionHeader: the row that names a group of rows (`NSTableView` group row, a source list's
//! header; design/30 section 2.6). One look; collapsible in a source list.

use crate::components::controls::disclosure::indicator;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::vocab::{Selection, Shown};

/// A group's name. The order is title, value, action. `collapse` makes the header a source
/// list's: a triangle before the title and a press anywhere on it asks for the flipped state
/// through its handler (the caller's `Shown` decides). `action_selection: Selected` draws the
/// action as a keyboard selection (`data-selected`): a command palette's cursor resting on a
/// group's "Show More". `on_action_mounted` hears the action's element as it mounts: the palette
/// keeps it in view when its cursor rests there.
#[component]
pub fn SectionHeader(
    title: String,
    #[props(default)] value: Option<String>,
    #[props(default)] action: Option<(String, EventHandler<()>)>,
    #[props(default)] action_selection: Selection,
    #[props(default)] on_action_mounted: Option<EventHandler<MountedEvent>>,
    #[props(default)] collapse: Option<(Shown, EventHandler<Shown>)>,
    #[props(default)] common: Common,
) -> Element {
    let selected = (action_selection == Selection::Selected).then_some("true");
    let data = common.data_attributes();
    let expanded = collapse.as_ref().map(|(shown, _)| shown.aria());
    let head = match collapse {
        Some((shown, on_toggle)) => rsx! {
            button {
                r#type: "button",
                class: "ds-section-header-toggle",
                "aria-expanded": shown.aria(),
                onclick: move |_| on_toggle.call(shown.flipped()),
                {indicator(shown)}
                span { class: "ds-section-header-title", "{title}" }
            }
        },
        None => rsx! {
            span { class: "ds-section-header-title", "{title}" }
        },
    };
    rsx! {
        div {
            class: common.class("ds-section-header"),
            id: common.id.clone(),
            role: "presentation",
            "aria-label": common.aria_label.clone(),
            "data-collapsible": expanded,
            onmounted: move |event| common.mounted(event),
            ..data,
            {head}
            if let Some(value) = value {
                span { class: "ds-section-header-value", "{value}" }
            }
            if let Some((label, onclick)) = action {
                button {
                    r#type: "button",
                    class: "ds-section-header-action",
                    "data-selected": selected,
                    onclick: move |_| onclick.call(()),
                    onmounted: move |event| {
                        if let Some(mounted) = on_action_mounted {
                            mounted.call(event);
                        }
                    },
                    "{label}"
                }
            }
        }
    }
}
