//! The read-only level bar: [`crate::Slider`] with `mode: SliderMode::Level` (sill FINDINGS Q74).
//! It is the slider's track and fill with nothing to take hold of: no thumb, not focusable, no
//! drag tracker and no key handler, so it can sit on a surface that takes no input (the OSD,
//! design/20 section 1.7). The fill's width transitions in `slider.css`.

use crate::components::slider::percent;
use crate::components::vocab::{Availability, Fraction};
use dioxus::prelude::*;

/// A level in a range, read-only: `role="progressbar"` with the same value attributes as the
/// control, so a screen reader hears the level it would hear from the slider.
#[component]
pub(crate) fn LevelBar(label: String, value: Fraction, availability: Availability) -> Element {
    let value = value.clamped();
    let fill = value.css();
    let now = percent(value);
    rsx! {
        div {
            class: "ds-slider",
            "data-mode": "level",
            role: "progressbar",
            "aria-label": "{label}",
            "aria-valuemin": "0",
            "aria-valuemax": "100",
            "aria-valuenow": "{now}",
            "aria-disabled": availability.aria_disabled(),
            style: "--f:{fill}",
            div { class: "ds-slider-track",
                div { class: "ds-slider-fill" }
            }
        }
    }
}
