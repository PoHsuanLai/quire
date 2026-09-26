//! The analog clock's vectors (design/23-WIDGETS.md sections 2 and 4.2; design/04-COMPONENTS.md
//! "Widgets"; sill FINDINGS Q183): four quarter marks; the thick round hour hand ending at .55
//! of the radius, the minute hand at .8 and the hub, drawn three times so they stand out of the
//! dial (a shade thrown down and right, a light thrown up and left, then the hands); and the
//! second hand on a vector of its own so its colour can differ. CSS does not reach inside an SVG
//! on Blitz (spike S6), so every paint is an attribute on `currentColor` and each vector takes its
//! colour from its own element. The shade and the light are shifted before the hands are
//! turned, so the light comes from the top left whichever way a hand points. `data-ds-svg`
//! tells the markup lint the vectors are quire's own.

use crate::components::clock_angles::{Hands, Tenths};
use crate::components::clock_kind::DayPhase;
use dioxus::prelude::*;

/// `rotate(a 50 50)`: a turn about the dial's centre in the 100-unit box.
fn turn(angle: Tenths) -> String {
    format!("rotate({} 50 50)", angle.css())
}

/// One drawing of the hands and the hub.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum HandLayer {
    /// The shade under them, down and to the right, a little wider.
    Shade,
    /// The light on them, up and to the left.
    Lit,
    /// The hands themselves.
    Hands,
}

/// How a layer is drawn: its class, its shift, and its hour, minute and hub widths.
#[derive(Debug, Clone, Copy, PartialEq)]
struct LayerDraw {
    class: &'static str,
    shift: &'static str,
    hour: &'static str,
    minute: &'static str,
    hub: &'static str,
}

impl HandLayer {
    /// Every layer, bottom first.
    pub(crate) const ALL: [HandLayer; 3] = [HandLayer::Shade, HandLayer::Lit, HandLayer::Hands];

    fn draw(self) -> LayerDraw {
        match self {
            HandLayer::Shade => LayerDraw {
                class: "ds-clock-shade",
                shift: "translate(1.6 2.2)",
                hour: "9.5",
                minute: "7.5",
                hub: "6.6",
            },
            HandLayer::Lit => LayerDraw {
                class: "ds-clock-lit",
                shift: "translate(-1.3 -1.3)",
                hour: "7.6",
                minute: "5.6",
                hub: "5.8",
            },
            HandLayer::Hands => LayerDraw {
                class: "ds-clock-hands",
                shift: "translate(0 0)",
                hour: "7",
                minute: "5",
                hub: "5.4",
            },
        }
    }
}

/// The four quarter marks.
pub(crate) fn marks_svg() -> Element {
    rsx! {
        svg { class: "ds-clock-marks", "data-ds-svg": "clock", view_box: "0 0 100 100", "aria-hidden": "true",
            for quarter in 0..4u16 {
                line {
                    key: "{quarter}",
                    x1: "50",
                    y1: "7",
                    x2: "50",
                    y2: "13",
                    stroke: "currentColor",
                    "stroke-width": "3.6",
                    "stroke-linecap": "round",
                    opacity: ".4",
                    transform: turn(Tenths(quarter * 900)),
                }
            }
        }
    }
}

/// The hour and minute hands and the hub as `layer`.
pub(crate) fn hands_svg(hands: Hands, layer: HandLayer) -> Element {
    let draw = layer.draw();
    rsx! {
        svg { class: draw.class, "data-ds-svg": "clock", view_box: "0 0 100 100", "aria-hidden": "true",
            g { transform: draw.shift,
                line {
                    x1: "50",
                    y1: "50",
                    x2: "50",
                    y2: "22.5",
                    stroke: "currentColor",
                    "stroke-width": draw.hour,
                    "stroke-linecap": "round",
                    transform: turn(hands.hour),
                }
                line {
                    x1: "50",
                    y1: "50",
                    x2: "50",
                    y2: "10",
                    stroke: "currentColor",
                    "stroke-width": draw.minute,
                    "stroke-linecap": "round",
                    transform: turn(hands.minute),
                }
                circle { cx: "50", cy: "50", r: draw.hub, fill: "currentColor" }
            }
        }
    }
}

/// The second hand and its own hub, over the others.
pub(crate) fn second_svg(angle: Tenths) -> Element {
    rsx! {
        svg { class: "ds-clock-second", "data-ds-svg": "clock", view_box: "0 0 100 100", "aria-hidden": "true",
            line {
                x1: "50",
                y1: "60",
                x2: "50",
                y2: "8",
                stroke: "currentColor",
                "stroke-width": "1.8",
                "stroke-linecap": "round",
                transform: turn(angle),
            }
            circle { cx: "50", cy: "50", r: "2.6", fill: "currentColor" }
        }
    }
}

/// The day or night mark beside a digital time: the sun a disc, the moon a crescent (a disc
/// with a second disc taken out of it).
pub(crate) fn phase_mark(phase: DayPhase) -> Element {
    let (class, shape) = match phase {
        DayPhase::Day => (
            "ds-clock-sun",
            rsx! { circle { cx: "12", cy: "12", r: "6.5", fill: "currentColor" } },
        ),
        DayPhase::Night => (
            "ds-clock-moon",
            rsx! { path { d: "M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z", fill: "currentColor" } },
        ),
    };
    rsx! {
        svg { class, "data-ds-svg": "clock", view_box: "0 0 24 24", "aria-hidden": "true", {shape} }
    }
}
