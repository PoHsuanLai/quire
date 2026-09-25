//! The mailo gaps 5 states, as data: the golden each renders to and how to make it.

use dioxus::prelude::*;
use ds::{Button, ButtonVariant, Run, RunTone, Text};

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

pub const CASES: &[Case] = &[
    Case {
        golden: "controls/button/label-runs.html",
        make: || rsx! { Button { variant: ButtonVariant::Quiet, label: quoted_head(), onclick: |_| {} } },
    },
    Case {
        golden: "controls/button/label-runs-named.html",
        make: || rsx! { Button { variant: ButtonVariant::Quiet, label: quoted_head(), aria_label: "Show the quoted message", onclick: |_| {} } },
    },
];
