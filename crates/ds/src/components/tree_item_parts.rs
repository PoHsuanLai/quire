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

/// The label: the caller's editing slot in its place while the place is renamed, else a button
/// that selects the place and keeps its press (so the folder does not also open or close) when
/// there is `onselect`, else words that toggle with the row.
pub(crate) fn label_part(
    label: Text,
    onselect: Option<EventHandler<Press>>,
    editing: Option<Element>,
) -> Element {
    if let Some(field) = editing {
        return editing_slot(field);
    }
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
///
/// `data-slot="trailing"` is the documented seam a consumer may select on (mailo gaps 7): its
/// own class inside the hover-revealed slot is styled as `[*|data-slot=trailing] .fold-more`,
/// which the stylesheet lint's `DsInternals` rule leaves alone, where `.ds-tree-item-trail
/// .fold-more` reaches into quire's internals.
pub(crate) fn trailing_slot(element: Element) -> Element {
    rsx! {
        span {
            class: "ds-tree-item-trail",
            "data-slot": "trailing",
            onclick: fence,
            {element}
        }
    }
}

/// The editing slot (mailo gaps 7): the caller's field (a Bare `TextInput` focused on mount,
/// its text selected) drawn where the label is, taking the label's free space so nothing on the
/// row moves. A press in it never reaches the summary, so it neither toggles nor selects the
/// row; the press's own pointer-down still puts the caret in the field (Blitz focuses a text
/// field on pointer-down, not in the click this fences). Keys are left alone: they start at the
/// field, so its Enter and Escape handlers hear them first.
fn editing_slot(field: Element) -> Element {
    rsx! {
        span {
            class: "ds-tree-item-label ds-tree-item-edit",
            "data-slot": "editing",
            onclick: fence,
            ondoubleclick: |event| event.stop_propagation(),
            onpointerup: |event| event.stop_propagation(),
            {field}
        }
    }
}

/// End a click at its slot: no summary toggle (the default), no row handler (propagation).
fn fence(event: MouseEvent) {
    event.stop_propagation();
    event.prevent_default();
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
