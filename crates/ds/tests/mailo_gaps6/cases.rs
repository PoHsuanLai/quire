//! The mailo gaps 6 states, as data: the golden each renders to and how to make it.

use dioxus::prelude::*;
use ds::{
    Button, ButtonVariant, DataAttr, DataName, Disclosure, DropState, ExtraClass, Here, Icon,
    IconButton, IconButtonVariant, PlaceId, Propagation, TreeItem, TreeShape,
};

/// One state and its golden.
pub struct Case {
    pub golden: &'static str,
    pub make: fn() -> Element,
}

/// `data-folder="<path>"`, as mailo's folder rows carry it.
fn folder(path: &str) -> Vec<DataAttr> {
    match DataName::parse("folder") {
        Ok(name) => vec![DataAttr::new(name, path)],
        Err(error) => panic!("{error}"),
    }
}

/// A consumer class, checked.
fn class(list: &str) -> Option<ExtraClass> {
    match ExtraClass::parse(list) {
        Ok(class) => Some(class),
        Err(error) => panic!("{error}"),
    }
}

/// The ⋯ for a folder row.
fn more(name: &str) -> Element {
    rsx! {
        IconButton { variant: IconButtonVariant::Strip, icon: Icon::Ellipsis, label: "Actions for {name}", propagation: Propagation::Stop, onclick: |_| {} }
    }
}

/// mailo's Projects folder with one subfolder, `open`, in `drop` state.
fn projects(open: Disclosure, drop: DropState) -> Element {
    rsx! {
        TreeItem {
            label: "Projects",
            open,
            on_toggle: |_| {},
            glyph: Icon::Folder,
            count: 3,
            trailing: more("Projects"),
            drop,
            place: PlaceId("INBOX/Projects".to_string()),
            onselect: |_| {},
            TreeItem { label: "Quire", open: Disclosure::Closed, on_toggle: |_| {}, shape: TreeShape::Leaf, glyph: Icon::Folder, place: PlaceId("INBOX/Projects/Quire".to_string()) }
        }
    }
}

pub const CASES: &[Case] = &[
    Case {
        golden: "controls/button/data-attr.html",
        make: || rsx! { Button { variant: ButtonVariant::Frame, label: "Receipts", data: folder("INBOX/Receipts"), onclick: |_| {} } },
    },
    Case {
        golden: "controls/button/extra-class.html",
        make: || rsx! { Button { variant: ButtonVariant::Mini, label: "Reply", extra_class: class("row-reveal  quiet-until-hover"), onclick: |_| {} } },
    },
    Case {
        golden: "controls/icon_button/data-attr-extra-class.html",
        make: || rsx! { IconButton { variant: IconButtonVariant::Strip, icon: Icon::Ellipsis, label: "Actions for Receipts", data: folder("INBOX/Receipts"), extra_class: class("fold-more"), propagation: Propagation::Stop, onclick: |_| {} } },
    },
    Case {
        golden: "lists/tree_item/open-idle.html",
        make: || projects(Disclosure::Open, DropState::Idle),
    },
    Case {
        golden: "lists/tree_item/open-accepts.html",
        make: || projects(Disclosure::Open, DropState::Accepts),
    },
    Case {
        golden: "lists/tree_item/open-target.html",
        make: || projects(Disclosure::Open, DropState::Target),
    },
    Case {
        golden: "lists/tree_item/open-source.html",
        make: || projects(Disclosure::Open, DropState::Source),
    },
    Case {
        golden: "lists/tree_item/closed-idle.html",
        make: || projects(Disclosure::Closed, DropState::Idle),
    },
    Case {
        golden: "lists/tree_item/closed-accepts.html",
        make: || projects(Disclosure::Closed, DropState::Accepts),
    },
    Case {
        golden: "lists/tree_item/closed-target.html",
        make: || projects(Disclosure::Closed, DropState::Target),
    },
    Case {
        golden: "lists/tree_item/closed-source.html",
        make: || projects(Disclosure::Closed, DropState::Source),
    },
    Case {
        golden: "lists/tree_item/leaf-current.html",
        make: || rsx! { TreeItem { label: "Receipts", open: Disclosure::Closed, on_toggle: |_| {}, shape: TreeShape::Leaf, here: Here::Current, count: 0, trailing: more("Receipts") } },
    },
];
