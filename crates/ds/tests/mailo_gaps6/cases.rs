//! The mail-app states, as data: the golden each renders to and how to make it.

use dioxus::prelude::*;
use ds::{
    Button, ButtonVariant, Common, DataAttr, DataName, DropState, ExtraClass, Icon, IconButton,
    IconButtonVariant, PlaceId, Propagation, RowState, Selection, Shown, TreeItem, TreeShape,
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
fn projects(open: Shown, drop: DropState) -> Element {
    rsx! {
        TreeItem {
            state: RowState { drop, ..RowState::default() },
            label: "Projects",
            open,
            on_toggle: |_| {},
            glyph: Icon::Folder,
            count: 3,
            trailing: more("Projects"),
            place: PlaceId("INBOX/Projects".to_string()),
            onselect: |_| {},
            TreeItem { label: "Quire", open: Shown::Hidden, on_toggle: |_| {}, shape: TreeShape::Leaf, glyph: Icon::Folder, place: PlaceId("INBOX/Projects/Quire".to_string()) }
        }
    }
}

pub const CASES: &[Case] = &[
    Case {
        golden: "controls/button/data-attr.html",
        make: || rsx! { Button { common: Common { data: folder("INBOX/Receipts"), ..Common::default() }, variant: ButtonVariant::Frame, label: "Receipts", onclick: |_| {} } },
    },
    Case {
        golden: "controls/button/extra-class.html",
        make: || rsx! { Button { common: Common { extra_class: class("row-reveal  quiet-until-hover"), ..Common::default() }, variant: ButtonVariant::Mini, label: "Reply", onclick: |_| {} } },
    },
    Case {
        golden: "controls/icon_button/data-attr-extra-class.html",
        make: || rsx! { IconButton { common: Common { data: folder("INBOX/Receipts"), extra_class: class("fold-more"), ..Common::default() }, variant: IconButtonVariant::Strip, icon: Icon::Ellipsis, label: "Actions for Receipts", propagation: Propagation::Stop, onclick: |_| {} } },
    },
    Case {
        golden: "lists/tree_item/open-idle.html",
        make: || projects(Shown::Visible, DropState::Idle),
    },
    Case {
        golden: "lists/tree_item/open-accepts.html",
        make: || projects(Shown::Visible, DropState::Accepts),
    },
    Case {
        golden: "lists/tree_item/open-target.html",
        make: || projects(Shown::Visible, DropState::Target),
    },
    Case {
        golden: "lists/tree_item/open-source.html",
        make: || projects(Shown::Visible, DropState::Source),
    },
    Case {
        golden: "lists/tree_item/closed-idle.html",
        make: || projects(Shown::Hidden, DropState::Idle),
    },
    Case {
        golden: "lists/tree_item/closed-accepts.html",
        make: || projects(Shown::Hidden, DropState::Accepts),
    },
    Case {
        golden: "lists/tree_item/closed-target.html",
        make: || projects(Shown::Hidden, DropState::Target),
    },
    Case {
        golden: "lists/tree_item/closed-source.html",
        make: || projects(Shown::Hidden, DropState::Source),
    },
    Case {
        golden: "lists/tree_item/leaf-current.html",
        make: || rsx! { TreeItem { state: RowState { selection: Selection::Selected, ..RowState::default() }, label: "Receipts", open: Shown::Hidden, on_toggle: |_| {}, shape: TreeShape::Leaf, count: 0, trailing: more("Receipts") } },
    },
];
