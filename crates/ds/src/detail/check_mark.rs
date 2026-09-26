//! CheckMark: a success check drawn on as `Settling::Drawing` says (design/26-DETAILS.md
//! section 3.2: "check = the tween drives `stroke-dashoffset` as an attribute", spike S6).

use super::settle::Settling;
use crate::icon::render::IconSize;
use crate::icon::stroke::stroke_width;
use crate::root::use_scale;
use dioxus::prelude::*;

/// The check's length on the 24-unit grid (`M20 6 9 17l-5-5`), for its dash.
const CHECK_LENGTH: f32 = 22.64;

/// The check as far as `settling` has drawn it: nothing unless it is `Settling::Drawing`, then
/// the stroke drawn so far, with the rest hidden by its dash offset. Decorative.
#[component]
pub fn CheckMark(settling: Settling, size: IconSize) -> Element {
    let Settling::Drawing(drawn) = settling else {
        return rsx! {};
    };
    let px = size.px();
    let stroke = stroke_width(size, use_scale());
    let share = f32::from(drawn.0.min(1000)) / 1000.0;
    let offset = CHECK_LENGTH * (1.0 - share);
    rsx! {
        svg {
            class: "ds-ic ds-check-mark",
            width: "{px}",
            height: "{px}",
            view_box: "0 0 24 24",
            "aria-hidden": "true",
            "stroke": "currentColor",
            "stroke-width": stroke,
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            "fill": "none",
            path {
                d: "M20 6 9 17l-5-5",
                "stroke-dasharray": "{CHECK_LENGTH}",
                "stroke-dashoffset": "{offset:.2}",
            }
        }
    }
}
