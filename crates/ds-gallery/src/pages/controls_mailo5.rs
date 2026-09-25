//! The Controls page's mailo gaps 5 buttons: a label of two runs (the composer's quoted-message
//! head) and a leading mark (the From dropdown's provider). Split from `controls.rs` to keep
//! that page under its size.

use super::{Section, Specimen};
use dioxus::prelude::*;
use ds::{
    Button, ButtonVariant, Expanded, Icon, Leading, MarkSize, MarkStyle, Provider, ProviderMark,
    Run, RunTone, Text, Trailing,
};

/// A provider's inline mark, for a From value.
fn mark(provider: Provider) -> Leading {
    Leading::Mark(rsx! {
        ProviderMark { provider, size: MarkSize::Inline, style: MarkStyle::Letter }
    })
}

/// The quoted message's head: who, strong; when, faint.
fn quoted_head() -> Text {
    Text::Runs(vec![
        Run::new("Dana Okafor", RunTone::Strong),
        Run::new(" wrote on Tue 22 Sep, 09:41", RunTone::Faint),
    ])
}

#[component]
pub fn ButtonsMailo5() -> Element {
    let mut open = use_signal(|| Expanded::Closed);
    rsx! {
        Section { title: "Button: a leading mark", note: "leading: Leading::Mark(element) puts a quire mark before the label: the From dropdown shows the account's provider inside its value. Leading::Glyph(icon) takes a glyph.",
            div { class: "g-row g-row-top",
                Specimen { name: "mark",
                    div { class: "g-row",
                        Button { variant: ButtonVariant::Quiet, label: "poh@acme.example", leading: mark(Provider::Google), trailing: Trailing::Caret, onclick: |_| {} }
                        Button { variant: ButtonVariant::Frame, label: "Local folders", leading: mark(Provider::Local), trailing: Trailing::Caret, onclick: |_| {} }
                        Button { variant: ButtonVariant::Mini, label: "Pinned", leading: Leading::Glyph(Icon::Pin), onclick: |_| {} }
                    }
                }
            }
        }
        Section { title: "Button: a label of runs", note: "label takes a Text: a String as before, or runs in their tones drawn inside the label's span (the composer's quoted-message head, who strong and when faint). The button is named by the runs' characters.",
            div { class: "g-row g-row-top",
                Specimen { name: "runs",
                    div { class: "g-row",
                        Button { variant: ButtonVariant::Quiet, label: quoted_head(), trailing: Trailing::Caret, expanded: open(),
                            onclick: move |_| open.set(match open() { Expanded::Open => Expanded::Closed, Expanded::Closed => Expanded::Open }) }
                        Button { variant: ButtonVariant::Mini, label: quoted_head(), onclick: |_| {} }
                    }
                }
            }
        }
    }
}
