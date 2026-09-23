//! Slider: a continuous value, divs and a drag tracker, because Blitz has no native range
//! (design/04-COMPONENTS.md section 5).

use crate::components::vocab::{Availability, Fraction};
use dioxus::prelude::*;

/// A value in a range.
#[component]
pub fn Slider(
    label: String,
    value: Fraction,
    #[props(default)] step: Fraction,
    #[props(default)] availability: Availability,
    onchange: EventHandler<Fraction>,
) -> Element {
    todo!()
}
