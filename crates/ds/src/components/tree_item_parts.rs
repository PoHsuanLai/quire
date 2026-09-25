//! The pieces of a `TreeItem` row: its attributes, its label, the leaf's chevron space and the
//! fenced trailing slot. Apart from the component so each stays short.

use crate::components::press::{Press, PressListeners, Propagation};
use crate::components::text_runs::{Text, text};
use crate::components::tree_item::Disclosure;
use dioxus::core::{Attribute, AttributeValue};
use dioxus::prelude::*;

/// The row's attributes, the same on a branch's `summary` and a leaf's `div`.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Row {
    /// `aria-current`.
    pub(crate) current: &'static str,
    /// `data-drop`: `target`, `accepts` or nothing.
    pub(crate) drop_attr: Option<&'static str>,
    /// `data-drag`: `source` or nothing.
    pub(crate) drag_attr: Option<&'static str>,
    /// `data-place`: the consumer's name for the place.
    pub(crate) place: Option<String>,
}

/// The label: a button that selects the place and keeps its press (so the folder does not also
/// open or close) when there is `onselect`, else words that toggle with the row.
pub(crate) fn label_part(label: Text, onselect: Option<EventHandler<Press>>) -> Element {
    match onselect {
        Some(select) => {
            let listen = PressListeners::new(select).with_propagation(Propagation::Stop);
            rsx! {
                button {
                    r#type: "button",
                    class: "ds-tree-item-label",
                    onclick: move |event| listen.click(&event),
                    oncontextmenu: move |event| listen.context_menu(&event),
                    onmouseup: move |event| listen.mouse_up(&event),
                    {text(&label)}
                }
            }
        }
        None => rsx! {
            span { class: "ds-tree-item-label", {text(&label)} }
        },
    }
}

/// A leaf's chevron: the same box, empty, so a leaf's label lines up with a branch's.
pub(crate) fn leaf_chevron() -> Element {
    rsx! {
        span { class: "ds-tree-item-chevron", "aria-hidden": "true" }
    }
}

/// The trailing slot. It keeps every press inside it from the summary: a button in it that
/// forgot `Propagation::Stop` still does not toggle the folder. The button hears its press
/// first (the event starts at the target), then the slot ends it.
pub(crate) fn trailing_slot(element: Element) -> Element {
    rsx! {
        span {
            class: "ds-tree-item-trail",
            onclick: move |event| {
                event.stop_propagation();
                event.prevent_default();
            },
            {element}
        }
    }
}

/// The `details`' `open`, in the null namespace, or no `open` at all.
///
/// Written by hand rather than as `open: bool` for two reasons found on Blitz. dioxus-native
/// puts an attribute it sets in the HTML namespace, and the user-agent sheet's
/// `details:not([open]) > :not(summary:first-of-type) { display:none !important }` matches only
/// a null-namespace `open`, so a `details` dioxus opened still hid its children (and an author
/// rule cannot override a user-agent `!important`). And dioxus-native writes a `false` as the
/// text "false", which is an `open` attribute all the same. An empty namespace is the null one,
/// and a `None` value removes the attribute. The value is "true" rather than empty because
/// dioxus-ssr, which writes `open` only when truthy, would drop an empty one.
pub(crate) fn open_attribute(open: Disclosure) -> Vec<Attribute> {
    let value = match open {
        Disclosure::Open => AttributeValue::Text("true".to_string()),
        Disclosure::Closed => AttributeValue::None,
    };
    vec![Attribute::new("open", value, Some(""), false)]
}
