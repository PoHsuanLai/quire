//! SectionHeader and Disclosure: the one header look with and without a value, an action or a
//! collapse, and the triangle that opens a body (design/30 sections 2.1 and 2.6).

use crate::pages::{Section, Specimen};
use dioxus::prelude::*;
use ds::components::controls::disclosure::Disclosure;
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;

/// The headers and the disclosures.
#[component]
pub fn HeaderGallery() -> Element {
    let mut source = use_signal(|| Shown::Visible);
    rsx! {
        Section {
            title: "SectionHeader",
            note: "One look. A value sits at the end, an action after it (a palette's cursor can rest on it), and in a source list the title is a button with a triangle.",
            div { class: "g-row g-row-top",
                Specimen { name: "Title", div { class: "g-list", style: "width:240px", SectionHeader { title: "Recent" } } }
                Specimen { name: "Value", div { class: "g-list", style: "width:240px", SectionHeader { title: "Grain", value: "12".to_string() } } }
                Specimen { name: "Action", div { class: "g-list", style: "width:240px", SectionHeader { title: "Files", action: Some(("Show More".to_string(), EventHandler::new(|()| {}))) } } }
                Specimen { name: "Action selected", div { class: "g-list", style: "width:240px",
                    SectionHeader { title: "Files", action: Some(("Show More".to_string(), EventHandler::new(|()| {}))), action_selection: Selection::Selected }
                } }
                Specimen { name: "Collapsible", div { class: "g-list", style: "width:240px",
                    SectionHeader { title: "Favourites", collapse: Some((source(), EventHandler::new(move |to| source.set(to)))) }
                } }
            }
        }
        Disclosures {}
    }
}

/// The triangle at each size, open and closed, busy, disabled, with a body.
#[component]
fn Disclosures() -> Element {
    let mut open = use_signal(|| Shown::Visible);
    let mut closed = use_signal(|| Shown::Hidden);
    rsx! {
        Section {
            title: "Disclosure",
            note: "The triangle turns a quarter over --t-quick; the body's height and opacity follow over --t-move from the height it measured, reversing mid-move if pressed again. Under Reduced motion both change at once.",
            div { class: "g-row g-row-top",
                Specimen { name: "Open", code: "shown: Visible".to_string(),
                    Disclosure { shown: open(), on_toggle: move |to| open.set(to), label: Some("Advanced".to_string()),
                        p { class: "g-note", "Three lines of settings that were folded away." }
                        p { class: "g-note", "The body keeps its own height." }
                    }
                }
                Specimen { name: "Closed", code: "shown: Hidden".to_string(),
                    Disclosure { shown: closed(), on_toggle: move |to| closed.set(to), label: Some("Details".to_string()),
                        p { class: "g-note", "Opens from its measured height." }
                    }
                }
                Specimen { name: "Disabled",
                    Disclosure { shown: Shown::Hidden, on_toggle: |_| {}, label: Some("Locked".to_string()), availability: Availability::Disabled }
                }
                Specimen { name: "Busy",
                    Disclosure { shown: Shown::Hidden, on_toggle: |_| {}, label: Some("Loading".to_string()), availability: Availability::Busy }
                }
                Specimen { name: "Sizes",
                    div { class: "g-col",
                        for size in [ControlSize::Mini, ControlSize::Small, ControlSize::Regular, ControlSize::Large] {
                            Disclosure { key: "{size:?}", shown: Shown::Hidden, on_toggle: |_| {}, label: Some(format!("{size:?}")), size }
                        }
                    }
                }
            }
        }
    }
}
