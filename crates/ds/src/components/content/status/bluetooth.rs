//! BluetoothGlyph: the rune, two connected dots and a slash as layers (design/26-DETAILS.md
//! 5.1.2). A connection that lands grows the dots in; turning the radio off draws the slash on.

use super::bluetooth_state::BluetoothState;
use super::part::{Paint, Part, Pen, Show, part_svg, slash_svg};
use super::slash::use_slash;
use crate::motion::detail::morph::Slashed;
use crate::style::icon::render::IconSize;
use crate::style::icon::shape::Shape;
use crate::style::icon::stroke::stroke_width;
use crate::style::scale::use_scale;
use dioxus::prelude::*;

/// Lucide `bluetooth`'s rune.
const RUNE: &[Shape] = &[Shape::Path("m7 7 10 10-5 5V2l5 5L7 17")];
/// The connected dots, either side of the rune's crossing.
const DOTS: &[Shape] = &[Shape::Path("M3.5 12h.01"), Shape::Path("M20.5 12h.01")];

/// The rune's and the dots' shows.
fn shows(state: BluetoothState) -> (Show, Show) {
    match state {
        BluetoothState::Off => (Show::Faint, Show::Hidden),
        BluetoothState::On | BluetoothState::Connecting(_) | BluetoothState::Failed(_) => {
            (Show::Lit, Show::Hidden)
        }
        BluetoothState::Connected => (Show::Lit, Show::Lit),
    }
}

fn slashed(state: BluetoothState) -> Slashed {
    match state {
        BluetoothState::Off => Slashed::On,
        BluetoothState::On
        | BluetoothState::Connecting(_)
        | BluetoothState::Failed(_)
        | BluetoothState::Connected => Slashed::Off,
    }
}

/// `span.ds-status-glyph[data-kind=bluetooth]`: the rune in `state` at `size`, in
/// `currentColor`. Decorative; put [`BluetoothState::words`] beside it (R8).
#[component]
pub fn BluetoothGlyph(
    state: BluetoothState,
    #[props(default = IconSize::Bar)] size: IconSize,
) -> Element {
    let drawn = use_slash(slashed(state));
    let pen = Pen {
        px: size.px(),
        stroke: stroke_width(size, use_scale()),
    };
    let (rune, dots) = shows(state);
    rsx! {
        span { class: "ds-status-glyph", "data-kind": "bluetooth", "aria-hidden": "true",
            {part_svg(Part { name: "rune", shapes: RUNE, paint: Paint::Stroke, show: rune }, &pen)}
            {part_svg(Part { name: "dots", shapes: DOTS, paint: Paint::Stroke, show: dots }, &pen)}
            {slash_svg(drawn, &pen)}
        }
    }
}
