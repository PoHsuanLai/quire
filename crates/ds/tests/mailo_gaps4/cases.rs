//! The mail-app states, as data: the golden each renders to and how to make it.

use dioxus::prelude::*;
use ds::Check;
use ds::{
    AccountFace, AccountTile, Button, ButtonFace, Colour, Common, Hex, Icon, MarkProvider,
    MarkSize, MarkStyle, ProviderMark, Trailing,
};
use ds::{Bezel, ControlSize};
use ds::{FieldBezel, FieldKind, TextField};

/// An account colour.
const SLATE: Colour = Colour::Solid(Hex([0x2f, 0x7f, 0x6e]));

/// A local-folders account: no provider, so the neutral folder mark.
fn local() -> AccountFace {
    AccountFace::One {
        initial: 'L',
        colour: SLATE,
        provider: MarkProvider::Local,
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
        make: || rsx! { AccountTile { account: local(), pressed: Check::On, unread: 2, onclick: |_| {} } },
    },
    Case {
        golden: "lists/provider_mark/local-row.html",
        make: || rsx! { ProviderMark { provider: MarkProvider::Local, size: MarkSize::Row, style: MarkStyle::Letter } },
    },
    Case {
        golden: "lists/provider_mark/local-image-ignored.html",
        make: || rsx! { ProviderMark { provider: MarkProvider::Local, size: MarkSize::Inline, style: MarkStyle::Image(ds::ImageSource("data:image/png;base64,iVBORw0KGgo=".to_string())) } },
    },
    Case {
        golden: "controls/button/frame.html",
        make: || rsx! { Button { bezel: Bezel::Toolbar, label: "Settings", icon: Some(Icon::Settings), onclick: |_| {} } },
    },
    Case {
        golden: "controls/button/frame-pressed.html",
        make: || rsx! { Button { bezel: Bezel::Toolbar, label: "Today", value: Some(Check::On), onclick: |_| {} } },
    },
    Case {
        golden: "controls/button/quiet-caret.html",
        make: || rsx! { Button { bezel: Bezel::Inline, label: "poh@acme.example", trailing: Trailing::Glyph(Icon::ChevronDown), shown: ds::Shown::Hidden, onclick: |_| {} } },
    },
    Case {
        golden: "controls/button/mini-trailing-glyph.html",
        make: || rsx! { Button { size: ControlSize::Mini, label: "Open", trailing: Trailing::Glyph(Icon::Link), onclick: |_| {} } },
    },
    Case {
        golden: "controls/button/face-bold.html",
        make: || rsx! { Button { bezel: Bezel::Inline, label: "Bold", face: ButtonFace::Bold, title: "Bold (Ctrl B)", value: Some(Check::On), onclick: |_| {} } },
    },
    Case {
        golden: "controls/button/face-italic.html",
        make: || rsx! { Button { bezel: Bezel::Inline, label: "Italic", face: ButtonFace::Italic, onclick: |_| {} } },
    },
    Case {
        golden: "controls/button/face-underline.html",
        make: || rsx! { Button { bezel: Bezel::Inline, label: "Underline", face: ButtonFace::Underline, onclick: |_| {} } },
    },
    Case {
        golden: "controls/button/face-strike-named.html",
        make: || rsx! { Button { common: Common { aria_label: Some("Strikethrough".to_string()), ..Common::default() }, bezel: Bezel::Inline, label: "Strike", face: ButtonFace::Strike, onclick: |_| {} } },
    },
    Case {
        golden: "controls/text_field/secret-value-unwritten.html",
        make: || rsx! { TextField { kind: FieldKind::Secure, label: "App password", value: "hunter2", placeholder: "App password", oninput: |_| {} } },
    },
    Case {
        golden: "controls/text_field/bare.html",
        make: || rsx! { TextField { bezel: FieldBezel::Plain, label: "Name", value: "Work", placeholder: "Name this Space", oninput: |_| {} } },
    },
];
