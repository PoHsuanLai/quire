//! ClockFace: a world clock's face (design/04-COMPONENTS.md "Widgets"; design/20 section 1.14;
//! sill FINDINGS Q183), an analog dial for the medium and large widgets or a digital row for
//! the small one, with the zone's name under it.
//!
//! The analog dial sits in a scope forced to the light scheme by day and the dark by night, so
//! its paper face and ink hands are the light pair by day and the dark pair by night whatever
//! the desktop's own scheme: a night clock reads as night on a light desktop too. The hand
//! angles are `clock_angles.rs`; the vectors `clock_dial.rs`.

use crate::appearance::Scheme;
use crate::components::bump_on::{bump_attrs, use_bump_on};
use crate::components::clock_angles::hands;
use crate::components::clock_dial::{hands_svg, second_svg};
use crate::components::clock_dial_depth::{self as depth, tapered_svg, ticks_svg};
use crate::components::clock_kind::{ClockLook, ClockTime, DayPhase, Seconds};
use crate::components::text_runs::{Text, text};
use crate::components::widget_looks::DialLook;
use crate::root::chrome::RootChrome;
use crate::root::env::use_env;
use crate::root::surface::Surface;
use dioxus::prelude::*;

/// The scheme a dial is drawn in by `phase`.
fn dial_scheme(phase: DayPhase) -> Scheme {
    match phase {
        DayPhase::Day => Scheme::Light,
        DayPhase::Night => Scheme::Dark,
    }
}

/// A clock showing `time`, tinted for `phase`, drawn as `look`, with `label` (the city) under
/// it. A digital face bumps once on each new minute; an analog one moves its hands. `dial`
/// builds the analog dial (design/23-WIDGETS.md section 4.2): the flat paper disc by default,
/// a bezel round a sky face, or a sky well; on a digital face a depth dial sets the time in
/// the display face with a small sky disc of the phase beside it.
#[component]
pub fn ClockFace(
    time: ClockTime,
    #[props(default)] phase: DayPhase,
    #[props(default)] look: ClockLook,
    #[props(into)] label: Text,
    #[props(default)] dial: DialLook,
) -> Element {
    let face = match (look, dial) {
        (ClockLook::Analog, DialLook::Paper) => rsx! { AnalogDial { time, phase } },
        (ClockLook::Analog, _) => rsx! { DepthDial { time, phase, dial } },
        (ClockLook::Digital, DialLook::Paper) => rsx! { Digits { time } },
        (ClockLook::Digital, _) => rsx! {
            span { class: "ds-clock-row",
                Digits { time }
                SkyDisc { phase }
            }
        },
    };
    rsx! {
        div {
            class: "ds-clock",
            "data-look": look.slug(),
            "data-dial": dial.attr(),
            "data-phase": phase.slug(),
            role: "img",
            "aria-label": "{label.plain_text()} {time.digits()}",
            {face}
            span { class: "ds-clock-label", {text(&label)} }
        }
    }
}

/// The dial, in its phase's scheme.
#[component]
fn AnalogDial(time: ClockTime, phase: DayPhase) -> Element {
    let material = use_env().material;
    let at = hands(time);
    let second = match time.second {
        Seconds::Shown(_) => Some(second_svg(at.second)),
        Seconds::Hidden => None,
    };
    rsx! {
        Surface { material, theme: dial_scheme(phase), chrome: RootChrome::Transparent,
            div { class: "ds-clock-dial",
                {hands_svg(at)}
                {second}
            }
        }
    }
}

/// A depth dial, in its phase's scheme: the face (a sky inside a bezel, or a sky well), the
/// indices, the hands over their shadow, the second hand, and the hub.
#[component]
fn DepthDial(time: ClockTime, phase: DayPhase, dial: DialLook) -> Element {
    let material = use_env().material;
    let at = hands(time);
    let shown = matches!(time.second, Seconds::Shown(_));
    rsx! {
        Surface { material, theme: dial_scheme(phase), chrome: RootChrome::Transparent,
            div { class: "ds-clock-dial",
                span { class: "ds-clock-face" }
                {ticks_svg(dial)}
                {tapered_svg("ds-clock-shade", at, shown)}
                {tapered_svg("ds-clock-hands", at, false)}
                if shown {
                    {depth::second_svg(at.second)}
                }
                span { class: "ds-clock-hub" }
            }
        }
    }
}

/// A small disc of the phase's sky, the day or night cue beside a digital time.
#[component]
fn SkyDisc(phase: DayPhase) -> Element {
    let material = use_env().material;
    rsx! {
        Surface { material, theme: dial_scheme(phase), chrome: RootChrome::Transparent,
            span { class: "ds-clock-sky" }
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
