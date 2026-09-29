//! A connected device's battery at the end of its settings row (design/26-DETAILS.md 5.2.3):
//! the battery glyph and its percentage, as text (design/30 section 1.3: a number changes
//! instantly).

use crate::components::content::status::{battery::BatteryGlyph, battery_state::BatteryState};
use crate::core::vocab::Fraction;
use crate::style::icon::render::IconSize;
use dioxus::prelude::*;

/// The glyph and the percentage, `84%` in tabular figures.
#[component]
pub(crate) fn RowBattery(level: Fraction) -> Element {
    let state = BatteryState {
        level: level.clamped(),
        ..BatteryState::default()
    };
    rsx! {
        BatteryGlyph { state, size: IconSize::Compact }
        span { class: "ds-settings-row-figure", "{level.whole_percent()}%" }
    }
}
