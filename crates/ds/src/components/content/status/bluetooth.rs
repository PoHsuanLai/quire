//! BluetoothGlyph: the stroked rune, two connected dots and a slash as layers (design/26-DETAILS.md
//! 5.1.2). A connection that lands grows the dots in; turning the radio off draws the slash on.

use super::bluetooth_state::BluetoothState;
use super::part::{Part, Pen, Show};
use super::slash::use_slash;
use super::stroked::{stroked_part_svg, stroked_slash_svg};
use dioxus::prelude::*;
use ds_motion::detail::morph::Slashed;
use ds_style::icon::render::IconSize;
use ds_style::icon::shape::Shape;
use ds_style::icon::stroke::stroke_width;
use ds_style::scale::use_scale;

/// Lucide `bluetooth`'s rune, stroked.
const RUNE: &[Shape] = &[Shape::Path("m7 7 10 10-5 5V2l5 5L7 17")];
/// The connected dots, either side of the rune's crossing, as stroked points.
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
            {stroked_part_svg(Part { name: "rune", shapes: RUNE, show: rune }, &pen)}
            {stroked_part_svg(Part { name: "dots", shapes: DOTS, show: dots }, &pen)}
            {stroked_slash_svg(drawn, &pen)}
        }
    }
}
