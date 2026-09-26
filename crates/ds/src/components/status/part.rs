//! The layers a status glyph is drawn from: one `svg` per part, stacked in an HTML wrapper, so the
//! stylesheet can fade, grow or tint each part over `--t-quick` (Blitz's stylesheet does not reach
//! inside an SVG, spike S6), and the Rust-driven parts (a slash drawn on, a battery's fill) are
//! recomputed per frame only while they move (design/26-DETAILS.md section 3.2).

use crate::components::vocab::Fraction;
use crate::detail::Lit;
use crate::icon::shape::Shape;
use dioxus::prelude::*;

/// How much of a part shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Show {
    /// Drawn in full ink.
    Lit,
    /// Drawn faint: a bar the signal does not reach, a radio that is off.
    Faint,
    /// Not drawn (it fades out and shrinks to .7 over `--t-quick`).
    Hidden,
}

impl Show {
    /// The `data-show` word.
    pub(crate) fn slug(self) -> &'static str {
        match self {
            Show::Lit => "lit",
            Show::Faint => "faint",
            Show::Hidden => "hidden",
        }
    }

    /// A pending loop's lit layer is `Lit`, an unlit one `Faint`.
    pub(crate) fn of(lit: Lit) -> Show {
        match lit {
            Lit::On => Show::Lit,
            Lit::Off => Show::Faint,
        }
    }
}

/// How a part is painted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Paint {
    /// Stroked on the Lucide grid: 2 units, round caps and joins.
    Stroke,
    /// Filled in the ink, no stroke: a small mark inside an outline (a bolt, a plug).
    Fill,
}

/// One part as drawn this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Part {
    /// The `data-part` word.
    pub(crate) name: &'static str,
    /// Its geometry on the 24 grid.
    pub(crate) shapes: &'static [Shape],
    /// Stroked or filled.
    pub(crate) paint: Paint,
    /// How much of it shows.
    pub(crate) show: Show,
}

/// A glyph's size and stroke, read once per render.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Pen {
    /// The side in logical pixels.
    pub(crate) px: u8,
    /// The stroke width attribute (snapped at fractional scales).
    pub(crate) stroke: String,
}

/// `part` as its own `svg.ds-ic.ds-status-part`.
pub(crate) fn part_svg(part: Part, pen: &Pen) -> Element {
    let (stroke, fill) = match part.paint {
        Paint::Stroke => ("currentColor", "none"),
        Paint::Fill => ("none", "currentColor"),
    };
    rsx! {
        svg {
            key: "{part.name}",
            class: "ds-ic ds-status-part",
            "data-part": part.name,
            "data-show": part.show.slug(),
            width: "{pen.px}",
            height: "{pen.px}",
            view_box: "0 0 24 24",
            "stroke": stroke,
            "stroke-width": pen.stroke.clone(),
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            "fill": fill,
            for shape in part.shapes {
                {shape_child(shape)}
            }
        }
    }
}

/// The slash's length on the 24 grid (`M2 2l20 20`), for its dash.
const SLASH_LENGTH: f32 = 28.29;

/// The slash drawn as far as `drawn` (thousandths), from the top left; nothing at 0.
pub(crate) fn slash_svg(drawn: Fraction, pen: &Pen) -> Element {
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

fn shape_child(shape: &Shape) -> Element {
    match shape {
        Shape::Path(d) => rsx! { path { d: "{d}" } },
        Shape::Circle { cx, cy, r } => rsx! { circle { cx: "{cx}", cy: "{cy}", r: "{r}" } },
        Shape::Rect {
            x,
            y,
            width,
            height,
            rx,
        } => rsx! {
            rect { x: "{x}", y: "{y}", width: "{width}", height: "{height}", rx: "{rx}" }
        },
    }
}
