//! The Bluetooth glyph's parts, drawn as strokes: the owner prefers the stroked rune to a filled
//! one (design/08-ICONS.md section 1.2, `Icon::ALWAYS_OUTLINE`). Same layers and `data-*` words
//! as `part`, with Lucide's 2 unit stroke, round caps and joins, and the slash drawn on by dash.

use super::part::{Part, Pen, Show};
use dioxus::prelude::*;
use ds_core::vocab::Fraction;
use ds_core::word::Word;
use ds_style::icon::shape::Shape;

/// The slash's length on the 24 grid (`M2 2l20 20`), for its dash.
const SLASH_LENGTH: f32 = 28.29;

/// `part` as its own stroked `svg.ds-ic.ds-status-part`.
pub(crate) fn stroked_part_svg(part: Part, pen: &Pen) -> Element {
    rsx! {
        svg {
            key: "{part.name}",
            class: "ds-ic ds-status-part",
            "data-part": part.name,
            "data-show": part.show.slug(),
            width: "{pen.px}",
            height: "{pen.px}",
            view_box: "0 0 24 24",
            "stroke": "currentColor",
            "stroke-width": pen.stroke.clone(),
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            "fill": "none",
            for shape in part.shapes {
                if let Shape::Path(d) = shape {
                    path { d: "{d}" }
                }
            }
        }
    }
}

/// The stroked slash drawn as far as `drawn` (thousandths); nothing at 0.
pub(crate) fn stroked_slash_svg(drawn: Fraction, pen: &Pen) -> Element {
    if drawn.0 == 0 {
        return rsx! {};
    }
    let offset = SLASH_LENGTH * (1.0 - f32::from(drawn.0.min(1000)) / 1000.0);
    rsx! {
        svg {
            class: "ds-ic ds-status-part",
            "data-part": "slash",
            "data-show": Show::Lit.slug(),
            width: "{pen.px}",
            height: "{pen.px}",
            view_box: "0 0 24 24",
            "stroke": "currentColor",
            "stroke-width": pen.stroke.clone(),
            "stroke-linecap": "round",
            "fill": "none",
            path {
                d: "M2 2l20 20",
                "stroke-dasharray": "{SLASH_LENGTH}",
                "stroke-dashoffset": "{offset:.2}",
            }
        }
    }
}
