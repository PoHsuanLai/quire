//! The mail-app states, as data: the golden each renders to and how to make it.

use crate::scoped::Scoped;
use dioxus::prelude::*;
use ds::{Accessory, Button, FieldFocus, Icon, Outline, Propagation, Row, RowLeading, Shown};
use ds::{Bezel, FieldBezel, ImagePosition, TextField};

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
        golden: "lists/row/outline-editing-leaf.html",
        make: || {
            rsx! {
                Scoped { Row { title: "Receipts", content: rename("Receipts"), leading: RowLeading::Icon(Icon::Folder), outline: Outline::Leaf } }
            }
        },
    },
];
