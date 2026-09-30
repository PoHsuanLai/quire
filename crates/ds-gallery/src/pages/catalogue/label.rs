//! Label: every role at every step of the type scale, disabled, runs in their tones and each severity.

use crate::pages::{Section, Specimen};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::content::text_runs::RunTone;
use ds::prelude::*;

/// The Label section.
#[component]
pub fn LabelSection() -> Element {
    let runs = TextLine::Runs(vec![
        TextRun::new("Dana Okafor", RunTone::Strong),
        TextRun::new(" wrote on Tue 22 Sep", RunTone::Faint),
    ]);
    rsx! {
        Section { title: "Label", note: "NSTextField label: four roles (the ink) at each step of the type scale; the only way to draw plain text.",
            for style in LabelStyle::ALL.iter().copied() {
                div { class: "g-row",
                    span { class: "g-name g-type-name", "{style.slug()}" }
                    for role in LabelRole::ALL.iter().copied() {
                        Label { text: format!("{} label", role.slug()), role, style }
                    }
                    Label { text: "disabled", style, availability: Availability::Disabled }
                }
            }
            div { class: "g-row",
                Specimen { name: "runs", Label { text: runs } }
            }
            div { class: "g-row",
                for severity in Severity::ALL.iter().copied() {
                    Specimen { name: format!("severity {}", severity.slug()),
                        Label { text: format!("{} status", severity.slug()), severity: Some(severity) }
                    }
                }
            }
        }
    }
}
