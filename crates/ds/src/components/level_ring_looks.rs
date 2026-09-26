//! The depth looks of a `LevelRing` (design/23-WIDGETS.md section 4.1): the ring in a groove
//! round a raised boss, and the horizontal cell. Every paint is the widget language's
//! (`WidgetPaint`); the vectors take their colour from their own element (spike S6).

use crate::components::vocab::Fraction;
use dioxus::prelude::*;

/// The level arc's radius for a circumference of 100 in the 36-unit box: `100 / 2π`.
const RADIUS: &str = "15.9155";

/// The groove, the liquid arc with its gloss, and the boss; `centre` (the glyph) on the boss.
pub(super) fn well(dash: Option<String>, centre: Element) -> Element {
    rsx! {
        span { class: "ds-ring-well" }
        if let Some(dash) = dash {
            {arc("ds-ring-arc", "4.4", dash.clone(), "rotate(-90 18 18) translate(18 18) scale(.9) translate(-18 -18)")}
            {arc("ds-ring-gloss", "1.5", dash, "rotate(-90 18 18) translate(18 18) scale(.96) translate(-18 -18)")}
        }
        span { class: "ds-ring-boss" }
        span { class: "ds-ring-centre", {centre} }
    }
}

/// One round-capped arc of the level on `currentColor`.
fn arc(class: &'static str, width: &'static str, dash: String, transform: &'static str) -> Element {
    rsx! {
        svg { class, "data-ds-svg": "ring", view_box: "0 0 36 36", "aria-hidden": "true",
            circle {
                cx: "18",
                cy: "18",
                r: RADIUS,
                fill: "none",
                stroke: "currentColor",
                "stroke-width": width,
                "stroke-linecap": "round",
                "stroke-dasharray": dash,
                transform,
            }
        }
    }
}

/// The cell: a recessed bed holding the liquid at `level`, the nub, and `centre` over both.
pub(super) fn cell(level: Fraction, centre: Element) -> Element {
    rsx! {
        span { class: "ds-ring-bed",
            if level.0 > 0 {
                span { class: "ds-ring-liquid", style: "--f:{level.css()}" }
            }
        }
        span { class: "ds-ring-nub" }
        span { class: "ds-ring-centre", {centre} }
    }
}
