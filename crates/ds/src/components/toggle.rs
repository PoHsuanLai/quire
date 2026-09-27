//! Toggle: a setting that is on or off and applies at once; a div track and knob, because
//! Blitz has no native checkbox (design/04-COMPONENTS.md section 4).
//!
//! The knob is driven motion (design/05 section 14, wave H1): a spring in Rust writes its
//! offset, `--knob-x` in pixels, so a second click mid-slide turns the knob back from where it is at
//! the speed it has, instead of restarting a transition. A click is a contact with no velocity:
//! critically damped, no bounce.

use crate::components::vocab::{Availability, Switch};
use crate::detail::Touch;
use crate::motion::{PxPerUnit, SpringResponse, SpringSpec, use_spring};
use dioxus::prelude::*;

/// The knob's travel, in pixels: the track's 30 px inside its padding less the 18 px knob.
const TRAVEL: f32 = 12.0;

/// Where the knob stands for `value`.
fn knob_at(value: Switch) -> f32 {
    match value {
        Switch::On => TRAVEL,
        Switch::Off => 0.0,
    }
}

/// An on/off switch.
#[component]
pub fn Toggle(
    label: String,
    value: Switch,
    #[props(default)] availability: Availability,
    onchange: EventHandler<Switch>,
) -> Element {
    // The contact behind the change this toggle asked for, spent only when the value it asked
    // for arrives; a value that changed from elsewhere moves remotely.
    let mut asked = use_signal(|| None::<(Switch, Touch)>);
    let touch = match *asked.peek() {
        Some((wanted, touch)) if wanted == value => touch,
        Some(_) | None => Touch::Remote,
    };
    let spec = SpringSpec::for_touch(touch).response(SpringResponse::Quick);
    let knob = use_spring(knob_at(value), spec, PxPerUnit(1.0));
    rsx! {
        button {
            r#type: "button",
            class: "ds-toggle",
            role: "switch",
            "aria-checked": value.aria(),
            "aria-label": "{label}",
            "aria-disabled": availability.aria_disabled(),
            style: "--knob-x:{knob.css()}",
            onclick: move |event| {
                if availability == Availability::Enabled {
                    let wanted = value.flipped();
                    asked.set(Some((wanted, Touch::from_event(&event))));
                    onchange.call(wanted);
                }
            },
            span { class: "ds-toggle-knob" }
        }
    }
}
