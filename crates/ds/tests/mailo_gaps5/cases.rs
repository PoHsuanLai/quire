//! The mailo gaps 5 states, as data: the golden each renders to and how to make it.

use dioxus::prelude::*;
use ds::components::vocab::{DropState, Here, PulseKey};
use ds::{
    Anim, Button, ButtonVariant, Icon, ItemKind, Leading, MarkSize, MarkStyle, PlaceId, Presence,
    Provider, ProviderMark, Run, RunTone, SidebarItem, Text, Trailing,
};

/// One state and its golden.
pub struct Case {
    pub golden: &'static str,
    pub make: fn() -> Element,
}

/// The composer's quoted-message head: who in the strong tone, when in the faint one.
fn quoted_head() -> Text {
    Text::Runs(vec![
        Run::new("Dana Okafor", RunTone::Strong),
        Run::new(" wrote on Tue 22 Sep, 09:41", RunTone::Faint),
    ])
}

/// The Archive place during a drag, in `drop` state.
fn archive(drop: DropState) -> Element {
    rsx! {
        SidebarItem {
            kind: ItemKind::Place { icon: Icon::Archive },
            label: "Archive",
            here: Here::Elsewhere,
            count: None,
            presence: Presence::Present,
            preview: None,
            pulse: PulseKey::rest(Anim::Gulp),
            onclick: |_| {},
            onclose: None,
            drop,
            place: PlaceId("archive".to_string()),
        }
    }
}

/// The From dropdown's provider, drawn inline.
fn google() -> Leading {
    Leading::Mark(
        rsx! { ProviderMark { provider: Provider::Google, size: MarkSize::Inline, style: MarkStyle::Letter } },
    )
}

pub const CASES: &[Case] = &[
    Case {
        golden: "controls/button/leading-mark.html",
        make: || rsx! { Button { variant: ButtonVariant::Quiet, label: "poh@acme.example", leading: google(), trailing: Trailing::Caret, expanded: ds::Expanded::Closed, onclick: |_| {} } },
    },
    Case {
        golden: "controls/button/leading-glyph.html",
        make: || rsx! { Button { variant: ButtonVariant::Mini, label: "Pinned", leading: Leading::Glyph(Icon::Pin), onclick: |_| {} } },
    },
    Case {
        golden: "lists/sidebar_item/place-drop-accepts.html",
        make: || archive(DropState::Accepts),
    },
    Case {
        golden: "controls/button/label-runs.html",
        make: || rsx! { Button { variant: ButtonVariant::Quiet, label: quoted_head(), onclick: |_| {} } },
    },
    Case {
        golden: "controls/button/label-runs-named.html",
        make: || rsx! { Button { variant: ButtonVariant::Quiet, label: quoted_head(), aria_label: "Show the quoted message", onclick: |_| {} } },
    },
];
