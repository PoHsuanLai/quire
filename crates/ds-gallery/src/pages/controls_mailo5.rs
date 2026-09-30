//! The Controls page's mail-app buttons, continued: a label of two runs (the composer's
//! quoted-message head) and a leading mark (the From dropdown's provider). Split from `controls.rs` to keep
//! that page under its size.

use super::{Section, Specimen};
use dioxus::prelude::*;
use ds::{Bezel, ControlSize};
use ds::{
    Button, Icon, Leading, MarkProvider, MarkStyle, ProviderMark, RunTone, Shown, TextLine,
    TextRun, Trailing,
};

/// A provider's inline mark, for a From value.
fn mark(provider: MarkProvider) -> Leading {
    Leading::Mark(rsx! {
        ProviderMark { provider, size: ControlSize::Small, style: MarkStyle::Letter }
    })
}

/// The quoted message's head: who, strong; when, faint.
fn quoted_head() -> TextLine {
    TextLine::Runs(vec![
        TextRun::new("Dana Okafor", RunTone::Strong),
        TextRun::new(" wrote on Tue 22 Sep, 09:41", RunTone::Faint),
    ])
}

#[component]
pub fn ButtonsMailo5() -> Element {
    let mut open = use_signal(|| Shown::Hidden);
    rsx! {
        Section { title: "Button: a leading mark", note: "leading: Leading::Mark(element) puts a quire mark before the label: the From dropdown shows the account's provider inside its value. Leading::Glyph(icon) takes a glyph.",
            div { class: "g-row g-row-top",
                Specimen { name: "mark",
                    div { class: "g-row",
                        Button { bezel: Bezel::Inline, label: "poh@acme.example", leading: mark(MarkProvider::Google), trailing: Trailing::Glyph(Icon::ChevronDown), onclick: |_| {} }
                        Button { bezel: Bezel::Toolbar, label: "Local folders", leading: mark(MarkProvider::Local), trailing: Trailing::Glyph(Icon::ChevronDown), onclick: |_| {} }
                        Button { size: ControlSize::Mini, label: "Pinned", leading: Leading::Glyph(Icon::Pin), onclick: |_| {} }
                    }
                }
            }
        }
        Section { title: "Button: a label of runs", note: "label takes a Text: a String as before, or runs in their tones drawn inside the label's span (the composer's quoted-message head, who strong and when faint). The button is named by the runs' characters.",
            div { class: "g-row g-row-top",
                Specimen { name: "runs",
                    div { class: "g-row",
                        Button { bezel: Bezel::Inline, label: quoted_head(), trailing: Trailing::Glyph(Icon::ChevronDown), shown: open(),
                            onclick: move |_| open.set(match open() { Shown::Visible => Shown::Hidden, Shown::Hidden => Shown::Visible }) }
                        Button { size: ControlSize::Mini, label: quoted_head(), onclick: |_| {} }
                    }
                }
            }
        }
    }
}
