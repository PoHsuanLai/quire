//! WifiGlyph: the Wi-Fi fan as layers, the dot and three arcs from the inside out, a "!" badge
//! and a slash (design/26-DETAILS.md 5.1.1). Joining lights one layer at a time, a step every
//! `--t-spin-step`, from the moment the join starts; a strength change cross-fades the arcs; no
//! internet grows the badge in; turning the radio off draws the slash on.

use super::part::{Part, Pen, Show, part_svg, slash_svg};
use super::slash::use_slash;
use super::wifi_state::{WifiReach, WifiState};
use dioxus::prelude::*;
use ds_motion::detail::{
    morph::Slashed,
    pending::{PendingFrame, PendingLayers, PendingSpec, PendingStyle},
    touch::Touch,
    use_detail::use_detail,
    use_operation::use_operation,
    use_pending::use_pending,
};
use ds_style::icon::render::IconSize;
use ds_style::icon::shape::Shape;
use ds_style::icon::solid_fan::{WIFI_DOT, WIFI_LARGE, WIFI_MID, WIFI_SMALL};
use ds_style::icon::stroke::stroke_width;
use ds_style::scale::use_scale;

/// The searching loop: the dot and the three arcs, one at a time (a step every `--t-spin-step`).
pub(crate) const SEARCHING: PendingSpec = PendingSpec {
    style: PendingStyle::Iterate,
    layers: PendingLayers(4),
};

/// The fan's parts, from the inside out (`ds_style::icon::solid_fan`), and the badge.
const DOT: &[Shape] = &[WIFI_DOT];
const ARC_1: &[Shape] = &[WIFI_SMALL];
const ARC_2: &[Shape] = &[WIFI_MID];
const ARC_3: &[Shape] = &[WIFI_LARGE];
/// The no-internet mark, a bar and a dot, in the fan's free lower-right corner.
const BADGE: &[Shape] = &[
    Shape::Rect {
        x: "20.5",
        y: "13.2",
        width: "2",
        height: "4.3",
        rx: "1",
    },
    Shape::Circle {
        cx: "21.5",
        cy: "20.6",
        r: "1.2",
    },
];

/// The layers' geometry and names, from the dot out.
const LAYERS: [(&str, &[Shape]); 4] = [
    ("dot", DOT),
    ("arc-1", ARC_1),
    ("arc-2", ARC_2),
    ("arc-3", ARC_3),
];

/// How each layer shows: the state as it is, or a pending frame over it.
pub(crate) fn layer_shows(state: WifiState, frame: PendingFrame) -> [Show; 4] {
    let each = |show: &dyn Fn(u8) -> Show| [0, 1, 2, 3].map(show);
    match (state, frame) {
        (WifiState::Joined { bars, .. }, _) => each(&|layer| lit_upto(layer, bars.top())),
        (WifiState::Joining(_), PendingFrame::Step(_)) => {
            each(&|layer| Show::of(frame.lit(SEARCHING, layer)))
        }
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

/// `span.ds-status-glyph[data-kind=wifi]`: the fan in `state` at `size`, in `currentColor`.
/// Decorative (`aria-hidden`); put [`WifiState::words`] beside it or in its label (R8). Bar
/// chrome is always there, so the first frame is still unless it is already joining (R1).
#[component]
pub fn WifiGlyph(state: WifiState, #[props(default = IconSize::Bar)] size: IconSize) -> Element {
    let detail = use_detail(state, Touch::Remote);
    let frame = use_pending(use_operation(detail.cue()), SEARCHING);
    let drawn = use_slash(slashed(state));
    let pen = Pen {
        px: size.px(),
        stroke: stroke_width(size, use_scale()),
    };
    let shows = layer_shows(state, frame);
    rsx! {
        span { class: "ds-status-glyph", "data-kind": "wifi",
            "data-state": slug(state), "aria-hidden": "true",
            for (index, (name, shapes)) in LAYERS.iter().enumerate() {
                {part_svg(Part { name, shapes, show: shows[index] }, &pen)}
            }
            {part_svg(Part { name: "badge", shapes: BADGE, show: badge(state) }, &pen)}
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
    use super::layer_shows;
    use crate::components::content::status::part::Show;
    use crate::components::content::status::wifi_state::{WifiBars, WifiReach, WifiState};
    use ds_motion::detail::{pending::PendingFrame, stamp::EventStamp};

    #[test]
    fn each_state_and_frame_shows_its_layers() {
        use Show::{Faint, Lit};
        let joined = |bars| WifiState::Joined {
            bars,
            reach: WifiReach::Internet,
        };
        let joining = WifiState::Joining(EventStamp(1));
        let cases: [(WifiState, PendingFrame, [Show; 4]); 6] = [
            (
                joined(WifiBars::One),
                PendingFrame::Idle,
                [Lit, Lit, Faint, Faint],
            ),
            (joined(WifiBars::Three), PendingFrame::Idle, [Lit; 4]),
            (WifiState::Off, PendingFrame::Idle, [Faint; 4]),
            (WifiState::Idle, PendingFrame::Idle, [Faint; 4]),
            // A join lights one layer a step, from the dot out, and starts at once.
            (joining, PendingFrame::Step(0), [Lit, Faint, Faint, Faint]),
            (joining, PendingFrame::Step(2), [Faint, Faint, Lit, Faint]),
        ];
        for (state, frame, want) in cases {
            assert_eq!(layer_shows(state, frame), want, "{state:?} {frame:?}");
        }
    }
}
