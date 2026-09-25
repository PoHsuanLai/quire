//! The analog clock's vectors (design/04-COMPONENTS.md "Widgets"; sill FINDINGS Q183): twelve
//! ticks, the hour and minute hands and the hub on `currentColor`, and the second hand on a
//! vector of its own so its colour can differ. CSS does not reach inside an SVG on Blitz (spike
//! S6), so every paint is an attribute and each vector takes its colour from its own element.
//! `data-ds-svg` tells the markup lint the vectors are quire's own.

use crate::components::clock_angles::{Hands, Tenths};
use dioxus::prelude::*;

/// The hours whose tick is drawn heavier: twelve, three, six and nine.
fn is_quarter(hour: u16) -> bool {
    hour.is_multiple_of(3)
}

/// `rotate(a 50 50)`: a turn about the dial's centre in the 100-unit box.
fn turn(angle: Tenths) -> String {
    format!("rotate({} 50 50)", angle.css())
}

/// The ticks, the hour and minute hands and the hub.
pub(crate) fn hands_svg(hands: Hands) -> Element {
    rsx! {
        svg {
            class: "ds-clock-hands",
            "data-ds-svg": "clock",
            view_box: "0 0 100 100",
            "aria-hidden": "true",
            for hour in 0..12u16 {
                line {
                    key: "{hour}",
                    x1: "50",
                    y1: "6",
                    x2: "50",
                    y2: if is_quarter(hour) { "14" } else { "11" },
                    stroke: "currentColor",
                    "stroke-width": if is_quarter(hour) { "3" } else { "2" },
                    "stroke-linecap": "round",
                    opacity: if is_quarter(hour) { ".7" } else { ".35" },
                    transform: turn(Tenths(hour * 300)),
                }
            }
            line {
                x1: "50",
                y1: "50",
                x2: "50",
                y2: "27",
                stroke: "currentColor",
                "stroke-width": "6",
                "stroke-linecap": "round",
                transform: turn(hands.hour),
            }
            line {
                x1: "50",
                y1: "50",
                x2: "50",
                y2: "13",
                stroke: "currentColor",
                "stroke-width": "4",
                "stroke-linecap": "round",
                transform: turn(hands.minute),
            }
            circle { cx: "50", cy: "50", r: "4.5", fill: "currentColor" }
        }
    }
}

/// The second hand and its own hub, over the others.
pub(crate) fn second_svg(angle: Tenths) -> Element {
    rsx! {
        svg {
            class: "ds-clock-second",
            "data-ds-svg": "clock",
            view_box: "0 0 100 100",
            "aria-hidden": "true",
            line {
                x1: "50",
                y1: "58",
                x2: "50",
                y2: "10",
                stroke: "currentColor",
                "stroke-width": "1.5",
                "stroke-linecap": "round",
                transform: turn(angle),
            }
            circle { cx: "50", cy: "50", r: "2.5", fill: "currentColor" }
        }
    }
}
