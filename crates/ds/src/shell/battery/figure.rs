//! A battery's percentage (design/23-WIDGETS.md section 4.1): the widget draws the number itself
//! (the ring draws none), so it is offered as a small component, [`BatteryFigure`], that writes
//! it as `93%` in tabular figures. A number changes instantly (design/30 section 1.3).

use crate::core::vocab::Fraction;
use dioxus::prelude::*;

/// `span.ds-battery-figure`: the percentage, `{n}%`, in tabular figures. Its size and face are
/// the caller's (a hero figure or a row's).
#[component]
pub fn BatteryFigure(level: Fraction) -> Element {
    rsx! {
        span { class: "ds-battery-figure", "{level.clamped().whole_percent()}%" }
    }
}
