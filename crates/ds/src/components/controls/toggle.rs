//! Toggle: a setting that is on or off and applies at once; a div track and knob, because
//! Blitz has no native checkbox (design/04-COMPONENTS.md section 4).
//!
//! The knob is driven motion (design/05 section 14): a spring in Rust writes its
//! offset, `--knob-x` in pixels, so a second click mid-slide turns the knob back from where it is at
//! the speed it has, instead of restarting a transition. A click is a contact with no velocity:
//! critically damped, no bounce.

use dioxus::prelude::*;
use ds_core::vocab::{Availability, Check};
use ds_core::word::Word;
use ds_motion::detail::touch::Touch;
use ds_motion::{
    spring_spec::{SpringResponse, SpringSpec},
    timeline::spring::PxPerUnit,
    use_spring::use_spring,
};
use ds_style::tokens::control_size::ControlSize;

/// Where the knob stands for `value` on a switch of `size`: off at the start, on at the end of
/// its travel (the track less its two knob insets and the knob, design/29-SIZING.md R3).
fn knob_at(value: Check, size: ControlSize) -> f32 {
    match value {
        Check::On => f32::from(size.scale().switch_travel().0),
        Check::Off | Check::Mixed => 0.0,
    }
}

/// A switch has no mixed look: `Mixed` is drawn, read out and flipped as `Off`.
fn refuse_mixed(value: Check) -> Check {
    match value {
        Check::Mixed => Check::Off,
        Check::On | Check::Off => value,
    }
}

/// An on/off switch, `size` on the ladder (Regular 38 x 22 when absent; Mini 26 x 15 in a
/// settings row).
#[component]
pub fn Toggle(
    label: String,
    value: Check,
    #[props(default)] size: ControlSize,
    #[props(default)] availability: Availability,
    onchange: EventHandler<Check>,
) -> Element {
    let value = refuse_mixed(value);
    // The contact behind the change this toggle asked for, spent only when the value it asked
    // for arrives; a value that changed from elsewhere moves remotely.
    let mut asked = use_signal(|| None::<(Check, Touch)>);
    let touch = match *asked.peek() {
        Some((wanted, touch)) if wanted == value => touch,
        Some(_) | None => Touch::Remote,
    };
    let spec = SpringSpec::for_touch(touch).response(SpringResponse::Quick);
    let knob = use_spring(knob_at(value, size), spec, PxPerUnit(1.0));
    rsx! {
        button {
            r#type: "button",
            class: "ds-toggle",
            role: "switch",
            "aria-checked": value.aria(),
            "data-size": size.slug(),
            "aria-label": "{label}",
            "aria-disabled": availability.aria_disabled(),
            "aria-busy": availability.aria_busy(),
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

#[cfg(test)]
mod tests {
    use super::{knob_at, refuse_mixed};
    use ds_core::vocab::Check;
    use ds_style::tokens::control_size::ControlSize;

    #[test]
    fn the_knob_travels_the_track_less_the_knob_and_its_insets() {
        const CASES: &[(ControlSize, f32)] = &[
            (ControlSize::Mini, 11.0),
            (ControlSize::Small, 14.0),
            (ControlSize::Regular, 16.0),
            (ControlSize::Large, 16.0),
        ];
        for (size, travel) in CASES {
            assert_eq!(knob_at(Check::On, *size), *travel, "{size:?}");
            assert_eq!(knob_at(Check::Off, *size), 0.0, "{size:?}");
        }
    }

    #[test]
    fn a_switch_refuses_mixed_and_draws_it_as_off() {
        const CASES: &[(Check, Check)] = &[
            (Check::On, Check::On),
            (Check::Off, Check::Off),
            (Check::Mixed, Check::Off),
        ];
        for &(given, drawn) in CASES {
            assert_eq!(refuse_mixed(given), drawn, "{given:?}");
        }
    }
}
