//! The mail-app states, as data: the golden each renders to and how to make it.

use crate::scoped::Scoped;
use dioxus::prelude::*;
use ds::components::controls::button_model::{Bezel, ImagePosition};
use ds::components::controls::press::Propagation;
use ds::components::lists::row::row::Outline;
use ds::prelude::*;

/// One state and its golden.
pub struct Case {
    pub golden: &'static str,
    pub make: fn() -> Element,
}

/// The rename field mailo draws in a row's label place.
fn rename(value: &str) -> Element {
    rsx! {
        TextField { bezel: FieldBezel::Plain, label: "Rename folder", value: value.to_string(), oninput: |_| {}, focus: FieldFocus::OnMount }
    }
}

/// The ⋯ for a folder row.
fn more(name: &str) -> Element {
    rsx! {
        Button { bezel: Bezel::Toolbar, image: ImagePosition::Only, icon: Icon::Ellipsis, label: "Actions for {name}", propagation: Propagation::Stop, onclick: |_| {} }
    }
}

pub const CASES: &[Case] = &[
    Case {
        golden: "lists/row/outline-editing-branch.html",
        make: || {
            rsx! {
                Scoped { Row {
                    title: "Projects",
                    content: rename("Projects"),
                    leading: RowLeading::Icon(Icon::Folder),
                    outline: Outline::Branch(Shown::Visible),
                    on_toggle: |_| {},
                    accessory: Accessory::Slot(more("Projects")),
                    Row { title: "Quire", leading: RowLeading::Icon(Icon::Folder), outline: Outline::Leaf }
                } }
            }
        },
    },
    Case {
        golden: "lists/row/edit-slot.html",
        make: || {
            rsx! {
                Scoped { Row { title: "Receipts", detail: "12 messages", edit: rename("Receipts"), leading: RowLeading::Icon(Icon::Folder), onclick: |_| {}, accessory: Accessory::Slot(more("Receipts")) } }
            }
        },
    },
    Case {
        golden: "lists/row/outline-editing-leaf.html",
        make: || {
            rsx! {
                Scoped { Row { title: "Receipts", content: rename("Receipts"), leading: RowLeading::Icon(Icon::Folder), outline: Outline::Leaf } }
            }
        },
    },
];
