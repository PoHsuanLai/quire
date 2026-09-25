//! The mailo gaps 6 states, as data: the golden each renders to and how to make it.

use dioxus::prelude::*;
use ds::{
    Button, ButtonVariant, DataAttr, DataName, ExtraClass, Icon, IconButton, IconButtonVariant,
    Propagation,
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
];
