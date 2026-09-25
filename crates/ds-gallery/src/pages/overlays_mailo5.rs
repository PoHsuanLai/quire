//! Overlays, mailo gaps 5: a scrim drawn inline in the pane it dims, under the reader the pane
//! peeks over it.

use super::Section;
use dioxus::prelude::*;
use ds::{Button, ButtonVariant, Flow, Scrim};

/// A pane with a list, dimmed by an inline scrim, and a reader drawn after the scrim above it.
#[component]
pub fn InlineScrim() -> Element {
    let mut peeked = use_signal(|| true);
    rsx! {
        Section {
            title: "Scrim drawn inline",
            note: "flow: Flow::Inline draws the scrim in the pane (position:absolute; inset:0 in the nearest positioned ancestor) at the pane's stacking level, so the reader the pane renders after it sits above it. A press on the veil dismisses.",
            div { class: "g-stage g-stage-tall",
                div { class: "g-stage-pad",
                    p { "Re: UIDL stability" }
                    p { "Invoice 2026-09" }
                    p { "Travel plans" }
                }
                if peeked() {
                    Scrim { label: "Close peek", flow: Flow::Inline, onclose: move |_| peeked.set(false) }
                    article { class: "g-peeked", "The peeked reader, above the veil." }
                } else {
                    div { class: "g-stage-pad",
                        Button { variant: ButtonVariant::Mini, label: "Peek again", onclick: move |_| peeked.set(true) }
                    }
                }
            }
        }
    }
}
