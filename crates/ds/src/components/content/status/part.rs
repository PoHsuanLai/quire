//! The layers a status glyph is drawn from: one `svg` per part, stacked in an HTML wrapper, so the
//! stylesheet can fade, grow or tint each part over `--t-quick` (Blitz's stylesheet does not reach
//! inside an SVG, spike S6), and the Rust-driven parts (a slash drawn on, a battery's fill) are
//! recomputed per frame only while they move (design/26-DETAILS.md section 3.2). Every part is
//! filled in the ink, no stroke (design/08-ICONS.md section 1.2).

use dioxus::prelude::*;
use ds_core::vocab::Fraction;
use ds_core::word::Word;
use ds_motion::detail::pending::Lit;
use ds_style::icon::posed::Thousandths;
use ds_style::icon::shape::Shape;
use ds_style::icon::slash::slash_mark;

/// How much of a part shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub(crate) enum Show {
    /// Drawn in full ink.
    Lit,
    /// Drawn faint: a bar the signal does not reach, a radio that is off.
    Faint,
    /// Not drawn (it fades out and shrinks to .7 over `--t-quick`).
    Hidden,
}

impl Show {
    /// A pending loop's lit layer is `Lit`, an unlit one `Faint`.
    pub(crate) fn of(lit: Lit) -> Show {
        match lit {
            Lit::On => Show::Lit,
            Lit::Off => Show::Faint,
        }
    }
}

/// One part as drawn this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Part {
    /// The `data-part` word.
    pub(crate) name: &'static str,
    /// Its geometry on the 24 grid.
    pub(crate) shapes: &'static [Shape],
    /// How much of it shows.
    pub(crate) show: Show,
}

/// A glyph's size, read once per render.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Pen {
    /// The side in logical pixels.
    pub(crate) px: u8,
    /// The stroke width attribute (snapped at fractional scales), for the stroked Bluetooth.
    pub(crate) stroke: String,
}

/// `part` as its own `svg.ds-ic.ds-status-part`.
pub(crate) fn part_svg(part: Part, pen: &Pen) -> Element {
    rsx! {
        svg {
            key: "{part.name}",
            class: "ds-ic ds-status-part",
            "data-part": part.name,
            "data-show": part.show.slug(),
            width: "{pen.px}",
            height: "{pen.px}",
            view_box: "0 0 24 24",
            "stroke": "none",
            "fill": "currentColor",
            for shape in part.shapes {
                {shape_child(shape)}
            }
        }
    }
}

/// The slash drawn as far as `drawn` (thousandths), from the top left: a solid bar; nothing at 0.
pub(crate) fn slash_svg(drawn: Fraction, pen: &Pen) -> Element {
    if drawn.0 == 0 {
        return rsx! {};
    }
    rsx! {
        svg {
            class: "ds-ic ds-status-part",
            "data-part": "slash",
            "data-show": Show::Lit.slug(),
            width: "{pen.px}",
            height: "{pen.px}",
            view_box: "0 0 24 24",
            "stroke": "none",
            "fill": "currentColor",
            {slash_mark(Thousandths(i32::from(drawn.0.min(1000))))}
        }
    }
}

fn shape_child(shape: &Shape) -> Element {
    match shape {
        Shape::Path(d) => rsx! { path { d: "{d}" } },
        Shape::Solid(d) => rsx! { path { d: "{d}", fill: "currentColor" } },
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
