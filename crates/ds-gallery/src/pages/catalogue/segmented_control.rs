//! SegmentedControl: SelectOne (the sliding thumb), SelectAny and Momentary at each size, with
//! image segments and a disabled segment.

use crate::pages::{Section, Specimen};
use dioxus::prelude::*;
use ds::components::controls::segmented::Tracking;
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;

/// The SegmentedControl section.
#[component]
pub fn SegmentedSection() -> Element {
    let mut one = use_signal(|| 1u8);
    let mut any = use_signal(|| vec![0u8, 2]);
    let mut last = use_signal(|| "none".to_owned());
    let words = || Choice::pairs([(0u8, "System"), (1, "Light"), (2, "Dark")]);
    let icons = || {
        vec![
            Choice::new(0u8, "Grid").with_icon(Icon::Grid),
            Choice::new(1u8, "List").with_icon(Icon::Columns),
            Choice::new(2u8, "Panel")
                .with_icon(Icon::Panel)
                .with_availability(Availability::Disabled),
        ]
    };
    // An image alone says nothing in words, so each is named for assistive technology.
    let image_only = || {
        vec![
            Choice::new(0u8, "").with_icon(Icon::Grid).with_name("Grid"),
            Choice::new(1u8, "").with_icon(Icon::Columns).with_name("Columns"),
            Choice::new(2u8, "").with_icon(Icon::Panel).with_name("Panel"),
        ]
    };
    rsx! {
        Section { title: "SegmentedControl", note: "NSSegmentedControl: SelectOne slides one thumb by a spring, SelectAny keeps each selected segment filled, Momentary fills the one held down; the arrows move a SelectOne choice and stop at the ends.",
            for size in ControlSize::ALL.iter().copied() {
                div { class: "g-row",
                    span { class: "g-name g-type-name", "{size.slug()}" }
                    SegmentedControl::<u8> { label: "View", choices: words(), tracking: Tracking::SelectOne(one()), size, onchange: move |next| one.set(next) }
                    SegmentedControl::<u8> { label: "Style", choices: words(), tracking: Tracking::SelectAny(any()), size, onchange: move |next| any.with_mut(|any| match any.iter().position(|v| *v == next) { Some(at) => { any.remove(at); } None => any.push(next) }) }
                    SegmentedControl::<u8> { label: "Step", choices: words(), tracking: Tracking::Momentary, size, onchange: move |next| last.set(format!("segment {next}")) }
                }
            }
            div { class: "g-row",
                Specimen { name: "image segments, one disabled".to_owned(),
                    SegmentedControl::<u8> { label: "Layout", choices: icons(), tracking: Tracking::SelectOne(one()), onchange: move |next| one.set(next) }
                }
                Specimen { name: "image only, named by Choice::with_name".to_owned(),
                    SegmentedControl::<u8> { label: "Layout", choices: image_only(), tracking: Tracking::SelectOne(one()), onchange: move |next| one.set(next) }
                }
                Specimen { name: "disabled".to_owned(),
                    SegmentedControl::<u8> { label: "View", choices: words(), tracking: Tracking::SelectOne(0), availability: Availability::Disabled, onchange: |_| {} }
                }
                Specimen { name: "busy".to_owned(),
                    SegmentedControl::<u8> { label: "View", choices: words(), tracking: Tracking::SelectOne(0), availability: Availability::Busy, onchange: |_| {} }
                }
                span { class: "g-code", "momentary: {last}" }
            }
        }
    }
}
