//! The mailo gaps 7 states, as data: the golden each renders to and how to make it.

use dioxus::prelude::*;
use ds::{
    Disclosure, FieldFace, Focus, Icon, IconButton, IconButtonVariant, PlaceId, Propagation,
    TextInput, TreeItem, TreeShape,
};

/// One state and its golden.
pub struct Case {
    pub golden: &'static str,
    pub make: fn() -> Element,
}

/// The rename field mailo draws in a row's label place.
fn rename(value: &str) -> Element {
    rsx! {
        TextInput { variant: FieldFace::Bare, label: "Rename folder", value: value.to_string(), oninput: |_| {}, focus: Focus::OnMount }
    }
}

/// The ⋯ for a folder row.
fn more(name: &str) -> Element {
    rsx! {
        IconButton { variant: IconButtonVariant::Strip, icon: Icon::Ellipsis, label: "Actions for {name}", propagation: Propagation::Stop, onclick: |_| {} }
    }
}

pub const CASES: &[Case] = &[
    Case {
        golden: "lists/tree_item/editing-branch.html",
        make: || {
            rsx! {
                TreeItem {
                    label: "Projects",
                    open: Disclosure::Open,
                    on_toggle: |_| {},
                    glyph: Icon::Folder,
                    count: 3,
                    onselect: |_| {},
                    editing: rename("Projects"),
                    trailing: more("Projects"),
                    place: PlaceId("INBOX/Projects".to_string()),
                    TreeItem { label: "Quire", open: Disclosure::Closed, on_toggle: |_| {}, shape: TreeShape::Leaf, glyph: Icon::Folder }
                }
            }
        },
    },
    Case {
        golden: "lists/tree_item/editing-leaf.html",
        make: || {
            rsx! {
                TreeItem { label: "Receipts", open: Disclosure::Closed, on_toggle: |_| {}, shape: TreeShape::Leaf, glyph: Icon::Folder, editing: rename("Receipts") }
            }
        },
    },
];
