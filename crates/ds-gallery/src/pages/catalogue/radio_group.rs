//! RadioGroup: a column, a row of image choices, a disabled choice and a busy group.

use crate::pages::{Section, Specimen};
use dioxus::prelude::*;
use ds::components::controls::radio_group::Arrangement;
use ds::prelude::*;
use ds_style::tokens::control_size::ControlSize;

/// The RadioGroup section.
#[component]
pub fn RadioGroupSection() -> Element {
    let mut size = use_signal(|| 1u8);
    let mut look = use_signal(|| 0u8);
    let sizes = || {
        vec![
            Choice::new(0u8, "Small"),
            Choice::new(1u8, "Medium"),
            Choice::new(2u8, "Large").with_availability(Availability::Disabled),
        ]
    };
    let looks = || {
        vec![
            Choice::new(0u8, "Light").with_icon(Icon::Sun),
            Choice::new(1u8, "Dark").with_icon(Icon::Moon),
            Choice::new(2u8, "Auto").with_icon(Icon::SunMoon),
        ]
    };
    rsx! {
        Section { title: "RadioGroup", note: "One of N: the arrows move to the next enabled item and check it, Home and End jump to the ends, Space checks the focused one; the checked item is the group's one tab stop.",
            div { class: "g-row g-row-top",
                for control in ControlSize::ALL.iter().copied() {
                    Specimen { name: control.slug().to_owned(),
                        RadioGroup::<u8> { label: "Size", choices: sizes(), value: size(), size: control, onchange: move |next| size.set(next) }
                    }
                }
            }
            div { class: "g-row g-row-top",
                Specimen { name: "row with images".to_owned(),
                    RadioGroup::<u8> { label: "Appearance", choices: looks(), value: look(), arrangement: Arrangement::Row, onchange: move |next| look.set(next) }
                }
                Specimen { name: "disabled".to_owned(),
                    RadioGroup::<u8> { label: "Size", choices: sizes(), value: 0, availability: Availability::Disabled, onchange: |_| {} }
                }
                Specimen { name: "busy".to_owned(),
                    RadioGroup::<u8> { label: "Size", choices: sizes(), value: 0, availability: Availability::Busy, onchange: |_| {} }
                }
            }
        }
    }
}
