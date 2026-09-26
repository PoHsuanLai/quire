//! ClockFace: a world clock's face (design/23-WIDGETS.md section 4.2; design/04-COMPONENTS.md
//! "Widgets"; sill FINDINGS Q183), an analog dial for the medium and large widgets or the time
//! as digits for the small one, with the zone's name under it.
//!
//! The analog dial is drawn in the soft language (design/23 section 2): a well pressed into the
//! plate, of the plate's own colour by day and ink-dark by night, four quarter marks, thick
//! round hands that stand out of it, a small hub and a thin accent second hand. The digital face sets the time in the display face and marks the phase with a sun or a moon
//! beside the city. The hand angles are `clock_angles.rs`; the vectors `clock_dial.rs`.

use crate::components::bump_on::{bump_attrs, use_bump_on};
use crate::components::clock_angles::hands;
use crate::components::clock_dial::{HandLayer, hands_svg, marks_svg, phase_mark, second_svg};
use crate::components::clock_kind::{ClockLook, ClockTime, DayPhase, Seconds};
use crate::components::text_runs::{Text, text};
use dioxus::prelude::*;

/// A clock showing `time`, for `phase`, drawn as `look`, with `label` (the city) under it. A
/// digital face bumps once on each new minute; an analog one moves its hands.
#[component]
pub fn ClockFace(
    time: ClockTime,
    #[props(default)] phase: DayPhase,
    #[props(default)] look: ClockLook,
    #[props(into)] label: Text,
) -> Element {
    let (face, mark) = match look {
        ClockLook::Analog => (rsx! { AnalogDial { time } }, None),
        ClockLook::Digital => (rsx! { Digits { time } }, Some(phase_mark(phase))),
    };
    rsx! {
        div {
            class: "ds-clock",
            "data-look": look.slug(),
            "data-phase": phase.slug(),
            role: "img",
            "aria-label": "{label.plain_text()} {time.digits()}",
            {face}
            span { class: "ds-clock-label",
                {mark}
                span { class: "ds-clock-city", {text(&label)} }
            }
        }
    }
}

/// The dial: a soft well in the plate, the quarter marks, the hands standing out of it, and
/// the second hand when shown.
#[component]
fn AnalogDial(time: ClockTime) -> Element {
    let at = hands(time);
    let second = match time.second {
        Seconds::Shown(_) => Some(second_svg(at.second)),
        Seconds::Hidden => None,
    };
    rsx! {
        div { class: "ds-clock-dial",
            {marks_svg()}
            for layer in HandLayer::ALL {
                {hands_svg(at, layer)}
            }
            {second}
        }
    }
}

/// The time as digits, bumping on each new minute.
#[component]
fn Digits(time: ClockTime) -> Element {
    let (class, alias) = bump_attrs("ds-clock-digits", use_bump_on(time.minute_key()));
    rsx! {
        span { class, "data-pulse": alias, "{time.digits()}" }
    }
}
