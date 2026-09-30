//! The mail-app states, as data: the golden each renders to and how to make it.

use dioxus::prelude::*;
use ds::{Bezel, ControlSize};
use ds::{
    Button, Common, DataAttr, DataName, DropState, Icon, Leading, MarkProvider, MarkSize,
    MarkStyle, ProviderMark, Row, RowLeading, RowState, RunTone, TextLine, TextRun, Trailing,
};

/// One state and its golden.
pub struct Case {
    pub golden: &'static str,
    pub make: fn() -> Element,
}

/// The composer's quoted-message head: who in the strong tone, when in the faint one.
fn quoted_head() -> TextLine {
    TextLine::Runs(vec![
        TextRun::new("Dana Okafor", RunTone::Strong),
        TextRun::new(" wrote on Tue 22 Sep, 09:41", RunTone::Faint),
    ])
}

/// The Archive place during a drag, in `drop` state.
fn archive(drop: DropState) -> Element {
    let place = DataName::parse("place")
        .map(|name| vec![DataAttr::new(name, "archive")])
        .unwrap_or_default();
    rsx! {
        Row {
            title: "Archive",
            leading: RowLeading::Icon(Icon::Archive),
            state: RowState { drop, ..RowState::default() },
            common: Common { data: place, ..Common::default() },
        }
    }
}

/// The From dropdown's provider, drawn inline.
fn google() -> Leading {
    Leading::Mark(
        rsx! { ProviderMark { provider: MarkProvider::Google, size: MarkSize::Inline, style: MarkStyle::Letter } },
    )
}

pub const CASES: &[Case] = &[
    Case {
        golden: "controls/button/leading-mark.html",
        make: || rsx! { Button { bezel: Bezel::Inline, label: "poh@acme.example", leading: google(), trailing: Trailing::Glyph(Icon::ChevronDown), shown: ds::Shown::Hidden, onclick: |_| {} } },
    },
    Case {
        golden: "controls/button/leading-glyph.html",
        make: || rsx! { Button { size: ControlSize::Mini, label: "Pinned", leading: Leading::Glyph(Icon::Pin), onclick: |_| {} } },
    },
    Case {
        golden: "lists/row/place-drop-accepts.html",
        make: || archive(DropState::Accepts),
    },
    Case {
        golden: "controls/button/label-runs.html",
        make: || rsx! { Button { bezel: Bezel::Inline, label: quoted_head(), onclick: |_| {} } },
    },
    Case {
        golden: "controls/button/label-runs-named.html",
        make: || rsx! { Button { common: Common { aria_label: Some("Show the quoted message".to_string()), ..Common::default() }, bezel: Bezel::Inline, label: quoted_head(), onclick: |_| {} } },
    },
];
