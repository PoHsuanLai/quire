//! Stepper: with its field and bare, at each size, at the ends of its range, disabled and busy.

use crate::pages::{Section, Specimen};
use dioxus::prelude::*;
use ds::components::fields::stepper::model::{Readout, StepRange};
use ds::components::fields::stepper::view::Stepper;
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;

/// The Stepper section.
#[component]
pub fn StepperSection() -> Element {
    let mut value = use_signal(|| 4i32);
    let mut bare = use_signal(|| 2i32);
    let range = StepRange::new(0, 10, 1);
    rsx! {
        Section { title: "Stepper", note: "NSStepper: the pair steps once on a press and repeats while it is held (after 500 ms, every 70 ms); with a field the typed number commits on Return or when the caret leaves, and the arrow keys step it. The half at the end of the range dims.",
            for size in ControlSize::ALL.iter().copied() {
                div { class: "g-row",
                    span { class: "g-name g-type-name", "{size.slug()}" }
                    Stepper { label: "Copies", value: value(), range, size, onchange: move |next| value.set(next) }
                    Stepper { label: "Copies", value: bare(), range, size, readout: Readout::Bare, onchange: move |next| bare.set(next) }
                    span { class: "g-code", "field {value()}, bare {bare()}" }
                }
            }
            div { class: "g-row",
                Specimen { name: "at the top".to_owned(),
                    Stepper { label: "Top", value: 10, range, onchange: |_| {} }
                }
                Specimen { name: "at the bottom".to_owned(),
                    Stepper { label: "Bottom", value: 0, range, onchange: |_| {} }
                }
                Specimen { name: "disabled".to_owned(),
                    Stepper { label: "Locked", value: 3, range, availability: Availability::Disabled, onchange: |_| {} }
                }
                Specimen { name: "busy".to_owned(),
                    Stepper { label: "Working", value: 3, range, availability: Availability::Busy, onchange: |_| {} }
                }
            }
        }
    }
}
