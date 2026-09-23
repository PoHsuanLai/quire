//! Toggle: a setting that is on or off and applies at once; a div track and knob, because
//! Blitz has no native checkbox (design/04-COMPONENTS.md section 4).

use crate::components::vocab::{Availability, Switch};
use dioxus::prelude::*;

/// An on/off switch.
#[component]
pub fn Toggle(
    label: String,
    value: Switch,
    #[props(default)] availability: Availability,
    onchange: EventHandler<Switch>,
) -> Element {
    todo!()
}
