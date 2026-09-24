//! The mailo gaps 2 states, as data: the golden each renders to and how to make it.

use dioxus::prelude::*;
use ds::components::vocab::Switch;
use ds::{
    AccountFace, AccountTile, AddAccountTile, Button, ButtonVariant, Colour, Expanded, Hex,
    ImageSource, InputVariant, MarkStyle, Provider, TextInput, TextInputKind,
};

/// An account colour.
const VIOLET: Colour = Colour::Solid(Hex([0x5b, 0x4f, 0xc4]));

/// Poh's account on Google.
fn poh() -> AccountFace {
    AccountFace::One {
        initial: 'P',
        colour: VIOLET,
        provider: Provider::Google,
        address: Some("poh@acme.example".to_string()),
    }
}

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
    // AccountTile: the favicon mark, and the Add account tile, with and without a hint.
    Case {
        golden: "lists/account_tile/one-image-mark.html",
        make: || rsx! { AccountTile { account: poh(), pressed: Switch::On, unread: 2, mark: MarkStyle::Image(ImageSource("data:image/png;base64,iVBORw0KGgo=".to_string())), onclick: |_| {} } },
    },
    Case {
        golden: "lists/account_tile/add.html",
        make: || rsx! { AddAccountTile { title: "Add account…", onclick: |_| {} } },
    },
    Case {
        golden: "lists/account_tile/add-named.html",
        make: || rsx! { AddAccountTile { label: "Add an account to Work", onclick: |_| {} } },
    },
];
