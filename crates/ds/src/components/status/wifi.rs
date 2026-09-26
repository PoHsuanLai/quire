//! WifiGlyph: the Wi-Fi fan as layers, the dot and three arcs from the inside out, a "!" badge
//! and a slash (design/26-DETAILS.md 5.1.1; G1-G5). Joining searches one layer at a time after
//! `PendingGrace` and holds a dimmed still frame at `PendingCap` (R4); a join that lands fills the
//! layers once up to the real bars; a strength change cross-fades the arcs; no internet grows the
//! badge in; a failed join shakes once; turning the radio off draws the slash on.

use super::part::{Paint, Part, Pen, Show, part_svg, slash_svg};
use super::slash::use_slash;
use super::wifi_state::{WifiReach, WifiState};
use crate::detail::{
    FirstShow, Layers, PendingFrame, PendingSpec, PendingStyle, SettleStyle, Settling, Slashed,
    Touch, use_detail, use_operation, use_pending, use_settle, use_shake,
};
use crate::icon::render::IconSize;
use crate::icon::shape::Shape;
use crate::icon::stroke::stroke_width;
use crate::root::use_scale;
use dioxus::prelude::*;

/// The searching loop: the dot and the three arcs, one at a time (1200 ms a cycle).
pub(crate) const SEARCHING: PendingSpec = PendingSpec {
    style: PendingStyle::Iterate,
    layers: Layers(4),
};

/// Lucide `wifi`'s parts, from the inside out, and the badge.
const DOT: &[Shape] = &[Shape::Path("M12 20h.01")];
const ARC_1: &[Shape] = &[Shape::Path("M8.5 16.429a5 5 0 0 1 7 0")];
const ARC_2: &[Shape] = &[Shape::Path("M5 12.859a10 10 0 0 1 14 0")];
const ARC_3: &[Shape] = &[Shape::Path("M2 8.82a15 15 0 0 1 20 0")];
/// The no-internet mark, in the fan's free lower-right corner.
const BADGE: &[Shape] = &[Shape::Path("M21.5 14v3.5"), Shape::Path("M21.5 21h.01")];

/// The layers' geometry and names, from the dot out.
const LAYERS: [(&str, &[Shape]); 4] = [
    ("dot", DOT),
    ("arc-1", ARC_1),
    ("arc-2", ARC_2),
    ("arc-3", ARC_3),
];

/// How each layer shows: the state as it is, a pending frame over it, or a success filling.
pub(crate) fn layer_shows(state: WifiState, frame: PendingFrame, settling: Settling) -> [Show; 4] {
    let each = |show: &dyn Fn(u8) -> Show| [0, 1, 2, 3].map(show);
    if let Settling::Filling(upto) = settling {
        return each(&|layer| lit_upto(layer, upto));
    }
    match (state, frame) {
        (WifiState::Joined { bars, .. }, _) => each(&|layer| lit_upto(layer, bars.top())),
        (WifiState::Joining(_), PendingFrame::Step(_)) => {
            each(&|layer| Show::of(frame.lit(SEARCHING, layer)))
        }
        (WifiState::Joining(_), PendingFrame::Stalled) => [Show::Lit; 4],
        (WifiState::Joining(_), PendingFrame::Idle)
        | (WifiState::Off | WifiState::Idle | WifiState::Failed(_), _) => [Show::Faint; 4],
    }
}

fn lit_upto(layer: u8, upto: u8) -> Show {
    if layer <= upto {
        Show::Lit
    } else {
        Show::Faint
    }
}

/// Whether the badge shows.
fn badge(state: WifiState) -> Show {
    match state {
        WifiState::Joined {
            reach: WifiReach::NoInternet,
            ..
        } => Show::Lit,
        WifiState::Joined {
            reach: WifiReach::Internet,
            ..
        }
        | WifiState::Off
        | WifiState::Idle
        | WifiState::Joining(_)
        | WifiState::Failed(_) => Show::Hidden,
    }
}

/// Whether the slash is on: only with the radio off.
fn slashed(state: WifiState) -> Slashed {
    match state {
        WifiState::Off => Slashed::On,
        WifiState::Idle
        | WifiState::Joining(_)
        | WifiState::Joined { .. }
        | WifiState::Failed(_) => Slashed::Off,
    }
}

/// How many layers a success fills: up to the real bars.
fn fill_layers(state: WifiState) -> Layers {
    match state {
        WifiState::Joined { bars, .. } => Layers(bars.top() + 1),
        WifiState::Off | WifiState::Idle | WifiState::Joining(_) | WifiState::Failed(_) => {
            Layers(4)
        }
    }
}

/// `span.ds-status-glyph[data-kind=wifi]`: the fan in `state` at `size`, in `currentColor`.
/// Decorative (`aria-hidden`); put [`WifiState::words`] beside it or in its label (R8). Bar
/// chrome is always there, so the first frame is still unless it is already joining (R1).
#[component]
pub fn WifiGlyph(state: WifiState, #[props(default = IconSize::Bar)] size: IconSize) -> Element {
    let detail = use_detail(state, FirstShow::Still, Touch::Remote);
    let frame = use_pending(use_operation(detail.cue()), SEARCHING);
    let settling = use_settle(detail.cue(), SettleStyle::Fill(fill_layers(state)));
    let shake = use_shake(detail.cue()).attrs();
    let drawn = use_slash(slashed(state));
    let pen = Pen {
        px: size.px(),
        stroke: stroke_width(size, use_scale()),
    };
    let shows = layer_shows(state, frame, settling);
    let (class, alias) = match shake {
        Some((anim, alias)) => (format!("ds-status-glyph {anim}"), Some(alias)),
        None => ("ds-status-glyph".to_owned(), None),
    };
    let still = match frame {
        PendingFrame::Stalled => "still",
        PendingFrame::Idle | PendingFrame::Step(_) => "idle",
    };
    rsx! {
        span { class, "data-pulse": alias, "data-kind": "wifi", "data-pending": still,
            "data-state": slug(state), "aria-hidden": "true",
            for (index, (name, shapes)) in LAYERS.iter().enumerate() {
                {part_svg(Part { name, shapes, paint: Paint::Stroke, show: shows[index] }, &pen)}
            }
            {part_svg(Part { name: "badge", shapes: BADGE, paint: Paint::Stroke, show: badge(state) }, &pen)}
            {slash_svg(drawn, &pen)}
        }
    }
}

/// The `data-state` word.
fn slug(state: WifiState) -> &'static str {
    match state {
        WifiState::Off => "off",
        WifiState::Idle => "idle",
        WifiState::Joining(_) => "joining",
        WifiState::Joined {
            reach: WifiReach::Internet,
            ..
        } => "joined",
        WifiState::Joined {
            reach: WifiReach::NoInternet,
            ..
        } => "no-internet",
        WifiState::Failed(_) => "failed",
    }
}

#[cfg(test)]
mod tests {
    use super::{Show, layer_shows};
    use crate::components::status::wifi_state::{WifiBars, WifiReach, WifiState};
    use crate::detail::{EventStamp, PendingFrame, Settling};

    #[test]
    fn each_state_frame_and_fill_shows_its_layers() {
        use Show::{Faint, Lit};
        let joined = |bars| WifiState::Joined {
            bars,
            reach: WifiReach::Internet,
        };
        let joining = WifiState::Joining(EventStamp(1));
        let rest = Settling::Rest;
        let cases: [(WifiState, PendingFrame, Settling, [Show; 4]); 8] = [
            (
                joined(WifiBars::One),
                PendingFrame::Idle,
                rest,
                [Lit, Lit, Faint, Faint],
            ),
            (joined(WifiBars::Three), PendingFrame::Idle, rest, [Lit; 4]),
            (WifiState::Off, PendingFrame::Idle, rest, [Faint; 4]),
            (WifiState::Idle, PendingFrame::Idle, rest, [Faint; 4]),
            // Inside the grace the join shows nothing yet; then one layer a step; held, whole.
            (joining, PendingFrame::Idle, rest, [Faint; 4]),
            (
                joining,
                PendingFrame::Step(2),
                rest,
                [Faint, Faint, Lit, Faint],
            ),
            (joining, PendingFrame::Stalled, rest, [Lit; 4]),
            // A success fills from the dot out, whatever the state says.
            (
                joined(WifiBars::Two),
                PendingFrame::Idle,
                Settling::Filling(1),
                [Lit, Lit, Faint, Faint],
            ),
        ];
        for (state, frame, settling, want) in cases {
            assert_eq!(
                layer_shows(state, frame, settling),
                want,
                "{state:?} {frame:?} {settling:?}"
            );
        }
    }
}
