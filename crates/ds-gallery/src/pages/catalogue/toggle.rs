//! Toggle: the switch at three sizes, off and on, disabled and busy; a live one springs its knob.

use crate::pages::{Section, Specimen};
use dioxus::prelude::*;
use ds::prelude::*;
use ds_style::tokens::control_size::ControlSize;

/// The Toggle section.
#[component]
pub fn ToggleSection() -> Element {
    let mut live = use_signal(|| Check::Off);
    rsx! {
        Section { title: "Toggle", note: "NSSwitch: the knob moves by a spring and the track tint fades over --t-quick; Space flips the focused one; a mixed value draws as off.",
            for size in [ControlSize::Mini, ControlSize::Small, ControlSize::Regular] {
                div { class: "g-row",
                    span { class: "g-name g-type-name", "{size.slug()}" }
                    Toggle { label: "Off", value: Check::Off, size, onchange: |_| {} }
                    Toggle { label: "On", value: Check::On, size, onchange: |_| {} }
                    Toggle { label: "Mixed", value: Check::Mixed, size, onchange: |_| {} }
                    Toggle { label: "Disabled", value: Check::On, size, availability: Availability::Disabled, onchange: |_| {} }
                    Toggle { label: "Busy", value: Check::Off, size, availability: Availability::Busy, onchange: |_| {} }
                }
            }
            div { class: "g-row",
                Specimen { name: "live".to_owned(),
                    Toggle { label: "Wi-Fi", value: live(), onchange: move |next| live.set(next) }
                }
            }
        }
    }
}
