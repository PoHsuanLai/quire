//! The mailo gaps 2 states, as data: the golden each renders to and how to make it.

use dioxus::prelude::*;
use ds::{Button, ButtonVariant, Expanded, InputVariant, TextInput, TextInputKind};

/// One state and the golden it must match (relative to `tests/snapshots`).
pub struct Case {
    pub golden: &'static str,
    pub make: fn() -> Element,
}

pub const CASES: &[Case] = &[
    // Button: a hint, a name for assistive technology, and the open state of what it opens.
    Case {
        golden: "controls/button/mini-titled-open.html",
        make: || rsx! { Button { variant: ButtonVariant::Mini, label: "+", title: "Add account…", aria_label: "Add account", expanded: Expanded::Open, onclick: |_| {} } },
    },
    Case {
        golden: "controls/button/quiet-closed.html",
        make: || rsx! { Button { variant: ButtonVariant::Quiet, label: "More", expanded: Expanded::Closed, onclick: |_| {} } },
    },
    // TextInput: a password, empty (the placeholder) and filled (the dots).
    Case {
        golden: "controls/text_input/password-empty.html",
        make: || rsx! { TextInput { variant: InputVariant::Boxed, kind: TextInputKind::Password, label: "Password", value: "", placeholder: "App password", oninput: |_| {} } },
    },
    Case {
        golden: "controls/text_input/password-filled.html",
        make: || rsx! { TextInput { variant: InputVariant::Boxed, kind: TextInputKind::Password, label: "Password", value: "hunter2", oninput: |_| {} } },
    },
];
