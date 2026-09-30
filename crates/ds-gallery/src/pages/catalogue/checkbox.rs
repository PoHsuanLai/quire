//! Checkbox: off, on and mixed at each size, disabled and busy.

use crate::pages::{Section, Specimen};
use dioxus::prelude::*;
use ds::Word;
use ds::{Availability, Check, Checkbox, ControlSize};

/// The Checkbox section.
#[component]
pub fn CheckboxSection() -> Element {
    let mut live = use_signal(|| Check::Mixed);
    rsx! {
        Section { title: "Checkbox", note: "NSButton checkbox: Space, or a press on the box or its label, flips it; a press on a mixed one turns it on.",
            for size in ControlSize::ALL.iter().copied() {
                div { class: "g-row",
                    span { class: "g-name g-type-name", "{size.slug()}" }
                    Checkbox { label: "Off", value: Check::Off, size, onchange: |_| {} }
                    Checkbox { label: "On", value: Check::On, size, onchange: |_| {} }
                    Checkbox { label: "Mixed", value: Check::Mixed, size, onchange: |_| {} }
                    Checkbox { label: "Disabled", value: Check::On, size, availability: Availability::Disabled, onchange: |_| {} }
                    Checkbox { label: "Busy", value: Check::Off, size, availability: Availability::Busy, onchange: |_| {} }
                }
            }
            div { class: "g-row",
                Specimen { name: "live".to_owned(),
                    Checkbox { label: "Include subfolders", value: live(), onchange: move |next| live.set(next) }
                }
            }
        }
    }
}
