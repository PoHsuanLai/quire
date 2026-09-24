//! The mailo gaps 4 states, as data: the golden each renders to and how to make it.

use dioxus::prelude::*;
use ds::components::vocab::Switch;
use ds::{
    AccountFace, AccountTile, Button, ButtonFace, ButtonVariant, Colour, Hex, Icon, MarkSize,
    MarkStyle, Provider, ProviderMark, Trailing,
};
use ds::{FieldFace, Grow, InputVariant, Rows, TextInput, TextInputKind};

/// An account colour.
const SLATE: Colour = Colour::Solid(Hex([0x2f, 0x7f, 0x6e]));

/// A local-folders account: no provider, so the neutral folder mark.
fn local() -> AccountFace {
    AccountFace::One {
        initial: 'L',
        colour: SLATE,
        provider: Provider::Local,
        address: None,
    }
}

/// One state and its golden.
pub struct Case {
    pub golden: &'static str,
    pub make: fn() -> Element,
}

pub const CASES: &[Case] = &[
    Case {
        golden: "lists/account_tile/one-local.html",
        make: || rsx! { AccountTile { account: local(), pressed: Switch::On, unread: 2, onclick: |_| {} } },
    },
    Case {
        golden: "lists/provider_mark/local-row.html",
        make: || rsx! { ProviderMark { provider: Provider::Local, size: MarkSize::Row, style: MarkStyle::Letter } },
    },
    Case {
        golden: "lists/provider_mark/local-image-ignored.html",
        make: || rsx! { ProviderMark { provider: Provider::Local, size: MarkSize::Inline, style: MarkStyle::Image(ds::ImageSource("data:image/png;base64,iVBORw0KGgo=".to_string())) } },
    },
    Case {
        golden: "controls/button/frame.html",
        make: || rsx! { Button { variant: ButtonVariant::Frame, label: "Settings", icon: Some(Icon::Settings), onclick: |_| {} } },
    },
    Case {
        golden: "controls/button/frame-pressed.html",
        make: || rsx! { Button { variant: ButtonVariant::Frame, label: "Today", pressed: Some(Switch::On), onclick: |_| {} } },
    },
    Case {
        golden: "controls/button/quiet-caret.html",
        make: || rsx! { Button { variant: ButtonVariant::Quiet, label: "poh@acme.example", trailing: Trailing::Caret, expanded: ds::Expanded::Closed, onclick: |_| {} } },
    },
    Case {
        golden: "controls/button/mini-trailing-glyph.html",
        make: || rsx! { Button { variant: ButtonVariant::Mini, label: "Open", trailing: Trailing::Glyph(Icon::Link), onclick: |_| {} } },
    },
    Case {
        golden: "controls/button/face-bold.html",
        make: || rsx! { Button { variant: ButtonVariant::Quiet, label: "Bold", face: ButtonFace::Bold, title: "Bold (Ctrl B)", pressed: Some(Switch::On), onclick: |_| {} } },
    },
    Case {
        golden: "controls/button/face-italic.html",
        make: || rsx! { Button { variant: ButtonVariant::Quiet, label: "Italic", face: ButtonFace::Italic, onclick: |_| {} } },
    },
    Case {
        golden: "controls/button/face-underline.html",
        make: || rsx! { Button { variant: ButtonVariant::Quiet, label: "Underline", face: ButtonFace::Underline, onclick: |_| {} } },
    },
    Case {
        golden: "controls/button/face-strike-named.html",
        make: || rsx! { Button { variant: ButtonVariant::Quiet, label: "Strike", face: ButtonFace::Strike, aria_label: "Strikethrough", onclick: |_| {} } },
    },
    Case {
        golden: "controls/text_input/secret-value-unwritten.html",
        make: || rsx! { TextInput { variant: InputVariant::Boxed, kind: TextInputKind::Secret, label: "App password", value: "hunter2", placeholder: "App password", oninput: |_| {} } },
    },
    Case {
        golden: "controls/text_input/file-empty.html",
        make: || rsx! { TextInput { variant: InputVariant::Boxed, kind: TextInputKind::File, label: "Signature image", value: "", placeholder: "No file chosen", oninput: |_| {}, on_pick: |()| {} } },
    },
    Case {
        golden: "controls/text_input/file-chosen.html",
        make: || rsx! { TextInput { variant: InputVariant::Boxed, kind: TextInputKind::File, label: "Signature image", value: "signature.png", oninput: |_| {}, on_pick: |()| {} } },
    },
    Case {
        golden: "controls/text_input/multiline-fixed.html",
        make: || rsx! { TextInput { variant: InputVariant::Boxed, kind: TextInputKind::Multiline { rows: Rows(3), grow: Grow::Fixed }, label: "Signature", value: "Poh\nAcme", placeholder: "Your signature", oninput: |_| {} } },
    },
    Case {
        golden: "controls/text_input/multiline-grown.html",
        make: || rsx! { TextInput { variant: InputVariant::Boxed, kind: TextInputKind::Multiline { rows: Rows(2), grow: Grow::ToContent }, label: "Signature", value: "a\nb\nc\nd", oninput: |_| {} } },
    },
    Case {
        golden: "controls/text_input/bare.html",
        make: || rsx! { TextInput { variant: FieldFace::Bare, label: "Name", value: "Work", placeholder: "Name this Space", oninput: |_| {} } },
    },
];
