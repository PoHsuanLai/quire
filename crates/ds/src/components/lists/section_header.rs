//! SectionHeader: the row that names a group of rows (`NSTableView` group row, a source list's
//! header; design/30 section 2.6). One look; collapsible in a source list.

use crate::components::controls::disclosure::indicator;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::vocab::{Selection, Shown};

/// One action at a header's end: its words and what a press does. `selection: Selected` draws it
/// as a keyboard selection (`data-selected`): a command palette's cursor resting on a group's
/// "Show More". `onmounted` hears its element as it mounts: the palette keeps it in view when
/// its cursor rests there.
#[derive(Debug, Clone, PartialEq)]
pub struct HeaderAction {
    /// The action's words: "Show More".
    pub label: String,
    /// A press on it.
    pub onclick: EventHandler<()>,
    /// Whether a keyboard cursor rests on it.
    pub selection: Selection,
    /// Hears its element as it mounts.
    pub onmounted: Option<EventHandler<MountedEvent>>,
}

impl HeaderAction {
    /// The action `label`, pressed through `onclick`, not selected and not watching its mount.
    pub fn new(label: impl Into<String>, onclick: EventHandler<()>) -> Self {
        HeaderAction {
            label: label.into(),
            onclick,
            selection: Selection::Unselected,
            onmounted: None,
        }
    }
}

/// A group's name. The order is title, value, actions. `collapse` makes the header a source
/// list's: a triangle before the title and a press anywhere on it asks for the flipped state
/// through its handler (the caller's `Shown` decides). `actions` are the buttons at the end, in
/// order ("Select All", "Clear"); one or several.
#[component]
pub fn SectionHeader(
    title: String,
    #[props(default)] value: Option<String>,
    #[props(default)] actions: Vec<HeaderAction>,
    #[props(default)] collapse: Option<(Shown, EventHandler<Shown>)>,
    #[props(default)] common: Common,
) -> Element {
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
            if !actions.is_empty() {
                span { class: "ds-section-header-actions",
                    for (index , action) in actions.into_iter().enumerate() {
                        button {
                            key: "{index}",
                            r#type: "button",
                            class: "ds-section-header-action",
                            "data-selected": (action.selection == Selection::Selected).then_some("true"),
                            onclick: move |_| action.onclick.call(()),
                            onmounted: move |event| {
                                if let Some(mounted) = action.onmounted {
                                    mounted.call(event);
                                }
                            },
                            "{action.label}"
                        }
                    }
                }
            }
        }
    }
}
