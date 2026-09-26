//! BluetoothGlyph: the rune, two connected dots and a slash as layers (design/26-DETAILS.md
//! 5.1.2; G6, G7). Connecting breathes the rune after `PendingGrace`, one step per
//! `--t-pending-step`, and holds it dimmed at `PendingCap` (R4); a connection that lands seals
//! once (`seal-out`: the service did it, not a press, R5) as the dots grow in; a failure shakes
//! once; turning the radio off draws the slash on.

use super::bluetooth_state::BluetoothState;
use super::part::{Paint, Part, Pen, Show, part_svg, slash_svg};
use super::slash::use_slash;
use crate::detail::{
    FirstShow, Layers, Lit, PendingFrame, PendingSpec, PendingStyle, SettleStyle, Settling,
    Slashed, Touch, use_detail, use_operation, use_pending, use_settle, use_shake,
};
use crate::icon::render::IconSize;
use crate::icon::shape::Shape;
use crate::icon::stroke::stroke_width;
use crate::root::use_scale;
use dioxus::prelude::*;

/// The connecting loop: the whole rune's opacity, high then low.
pub(crate) const BREATHING: PendingSpec = PendingSpec {
    style: PendingStyle::Breathe,
    layers: Layers(1),
};

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

/// The glyph's `data-pending` word: `low` on a breath's low half, `still` when held.
pub(crate) fn breath(frame: PendingFrame) -> &'static str {
    match frame {
        PendingFrame::Idle => "idle",
        PendingFrame::Stalled => "still",
        PendingFrame::Step(_) => match frame.lit(BREATHING, 0) {
            Lit::On => "high",
            Lit::Off => "low",
        },
    }
}

/// `span.ds-status-glyph[data-kind=bluetooth]`: the rune in `state` at `size`, in
/// `currentColor`. Decorative; put [`BluetoothState::words`] beside it (R8).
#[component]
pub fn BluetoothGlyph(
    state: BluetoothState,
    #[props(default = IconSize::Bar)] size: IconSize,
) -> Element {
    let detail = use_detail(state, FirstShow::Still, Touch::Remote);
    let frame = use_pending(use_operation(detail.cue()), BREATHING);
    let settling = use_settle(detail.cue(), SettleStyle::LockIn);
    let shake = use_shake(detail.cue()).attrs();
    let drawn = use_slash(slashed(state));
    let seal = match settling {
        Settling::Sealing(key) => key.attrs(),
        Settling::Rest | Settling::Filling(_) | Settling::Drawing(_) => None,
    };
    let (class, alias) = match shake.or(seal) {
        Some((anim, alias)) => (format!("ds-status-glyph {anim}"), Some(alias)),
        None => ("ds-status-glyph".to_owned(), None),
    };
    let pen = Pen {
        px: size.px(),
        stroke: stroke_width(size, use_scale()),
    };
    let (rune, dots) = shows(state);
    rsx! {
        span { class, "data-pulse": alias, "data-kind": "bluetooth", "data-pending": breath(frame),
            "aria-hidden": "true",
            {part_svg(Part { name: "rune", shapes: RUNE, paint: Paint::Stroke, show: rune }, &pen)}
            {part_svg(Part { name: "dots", shapes: DOTS, paint: Paint::Stroke, show: dots }, &pen)}
            {slash_svg(drawn, &pen)}
        }
    }
}
