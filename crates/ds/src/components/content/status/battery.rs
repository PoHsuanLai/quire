//! BatteryGlyph: a battery outline with a continuous fill layer, a bolt and a plug as their own
//! layers (design/26-DETAILS.md 5.1.3). The fill follows its level linearly over `--t-move`
//! (design/30 section 1.3), and stands at its level on mount; plugging in shows the bolt (the plug
//! when the charger holds it), the fill dimming under the mark; crossing the low threshold
//! cross-fades the fill to `--battery-low` (R15). No count, no pulse (R12).

use super::battery_state::{BatteryPower, BatteryState, Tone};
use super::part::{Paint, Part, Pen, Show, part_svg};
use crate::motion::detail::tween::{TweenSpec, use_tween};
use dioxus::prelude::*;
use ds_core::word::Word;
use ds_style::icon::render::IconSize;
use ds_style::icon::shape::Shape;
use ds_style::icon::stroke::stroke_width;
use ds_style::scale::use_scale;
use ds_style::tokens::{easing::EasingToken, timing::DurationToken};

/// Lucide `battery`'s outline: the body and the terminal.
const OUTLINE: &[Shape] = &[
    Shape::Rect {
        x: "2",
        y: "6",
        width: "16",
        height: "12",
        rx: "2",
    },
    Shape::Path("M22 14v-4"),
];

/// A filled bolt centred in the body.
const BOLT: &[Shape] = &[Shape::Path("M11 7.5 7.5 12.5h2.25L9 16.5l3.5-5h-2.25z")];

/// A filled plug centred in the body: two prongs, the head, the cord.
const PLUG: &[Shape] = &[
    Shape::Rect {
        x: "8",
        y: "8",
        width: "1",
        height: "2",
        rx: "0",
    },
    Shape::Rect {
        x: "11",
        y: "8",
        width: "1",
        height: "2",
        rx: "0",
    },
    Shape::Path("M7.5 10h5v1.75a2.5 2.5 0 0 1-5 0z"),
    Shape::Rect {
        x: "9.5",
        y: "13.5",
        width: "1",
        height: "2.5",
        rx: "0",
    },
];

/// How the fill follows its level: a determinate value moves linearly over `--t-move`.
const FILL: TweenSpec = TweenSpec {
    duration: DurationToken::Move,
    easing: EasingToken::Linear,
};

/// The fill's box inside the body, on the 24 grid: a unit of air inside the stroke.
const FILL_X: f32 = 4.5;
const FILL_Y: f32 = 8.5;
const FILL_WIDTH: f32 = 11.0;
const FILL_HEIGHT: f32 = 7.0;

/// Which mark shows, and whether the fill dims under it.
fn marks(power: BatteryPower) -> (Show, Show, &'static str) {
    match power {
        BatteryPower::Battery => (Show::Hidden, Show::Hidden, "none"),
        BatteryPower::Charging => (Show::Lit, Show::Hidden, "mark"),
        BatteryPower::Held => (Show::Hidden, Show::Lit, "mark"),
    }
}

fn tone_slug(tone: Tone) -> &'static str {
    match tone {
        Tone::Normal => "normal",
        Tone::Low => "low",
    }
}

/// `span.ds-status-glyph[data-kind=battery]`: the battery in `state` at `size`, in
/// `currentColor` (the fill in `--battery-low` when low). Decorative; put
/// [`BatteryState::words`] beside it or in its label (R8).
#[component]
pub fn BatteryGlyph(
    state: BatteryState,
    #[props(default = IconSize::Bar)] size: IconSize,
) -> Element {
    let share = use_tween(state.drawn(), FILL);
    let look = state.look();
    let pen = Pen {
        px: size.px(),
        stroke: stroke_width(size, use_scale()),
    };
    let (bolt, plug, under) = marks(look.power);
    let width = FILL_WIDTH * f32::from(share.0.min(1000)) / 1000.0;
    rsx! {
        span { class: "ds-status-glyph", "data-kind": "battery", "aria-hidden": "true",
            {part_svg(Part { name: "outline", shapes: OUTLINE, paint: Paint::Stroke, show: Show::Lit }, &pen)}
            svg {
                class: "ds-ic ds-status-part",
                "data-part": "fill",
                "data-show": Show::Lit.slug(),
                "data-tone": tone_slug(look.tone),
                "data-under": under,
                width: "{pen.px}",
                height: "{pen.px}",
                view_box: "0 0 24 24",
                "stroke": "none",
                "fill": "currentColor",
                if width > 0.0 {
                    rect { x: "{FILL_X}", y: "{FILL_Y}", width: "{width:.2}", height: "{FILL_HEIGHT}", rx: "1" }
                }
            }
            {part_svg(Part { name: "bolt", shapes: BOLT, paint: Paint::Fill, show: bolt }, &pen)}
            {part_svg(Part { name: "plug", shapes: PLUG, paint: Paint::Fill, show: plug }, &pen)}
        }
    }
}
