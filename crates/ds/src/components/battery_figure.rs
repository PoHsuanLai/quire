//! A battery's percentage counting up with its ring (design/23-WIDGETS.md section 4.1): the
//! widget draws the number itself (the ring draws none), so the count the ring's fill reaches
//! each frame is offered as a hook, [`use_battery_figure`], and as a small component,
//! [`BatteryFigure`], that writes it as `93%` in tabular figures.
//!
//! Given the same `level` and `wake` as its [`crate::BatteryLevel`], the count moves in step with
//! the arc: both read [`use_battery_fill`], whose frames are a pure function of the time since
//! the same effect started. It is always a whole percent and ends exactly on the true one.

use crate::components::battery_level::{percent_of, use_battery_fill};
use crate::components::vocab::Fraction;
use crate::motion::WakeStamp;
use dioxus::prelude::*;

/// The percentage a battery at `level` shows this frame: counting from 0 on mount and on each
/// new `wake`, from the old percentage on each new `level`, at once under Reduced motion.
pub fn use_battery_figure(level: Fraction, wake: WakeStamp) -> u16 {
    percent_of(use_battery_fill(level, wake).shown)
}

/// `span.ds-battery-figure`: the counting percentage, `{n}%`, in tabular figures so the width
/// holds while it counts. Its size and face are the caller's (a hero figure or a row's).
#[component]
pub fn BatteryFigure(level: Fraction, #[props(default)] wake: WakeStamp) -> Element {
    let figure = use_battery_figure(level, wake);
    rsx! {
        span { class: "ds-battery-figure", "{figure}%" }
    }
}
