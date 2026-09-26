//! Persona: the user's own animated character, drawn from simple flat shapes
//! (design/24-PERSONA.md). A [`PersonaSpec`] says what it looks like; the caller sets its
//! [`Mood`] and the component plays it. [`UserPicture`] lets a surface take either a letter
//! [`crate::AvatarFace`] or a persona.

mod face;
mod geometry;
mod head;
mod life;
mod mark;
pub mod mood;
mod palette;
mod picture;
mod seed;
pub mod spec;
#[cfg(test)]
mod tests;

pub use mood::{Mood, PersonaFinish, PersonaSize, WakeStamp};
pub(crate) use picture::photo;
pub use picture::{UserPicture, UserPortrait};
pub use seed::{BLINK_MAX, BLINK_MIN};
pub use spec::{
    Accessory, Backdrop, Brows, Cheeks, Creature, Eyes, HairTone, HeadShape, Mouth, PersonaSeed,
    PersonaSpec, Tone, Top,
};

use crate::components::bump_on::bump_attrs;
use crate::components::muted::desaturated;
use crate::root::env::use_env;
use crate::tokens::Hex;
use dioxus::prelude::*;
use geometry::num;
use mark::Mark;

/// `hex` in `finish`.
fn finished(hex: String, finish: PersonaFinish) -> String {
    match (finish, Hex::parse(&hex)) {
        (PersonaFinish::Muted, Some(parsed)) => desaturated(parsed).css(),
        (PersonaFinish::Colour, _) | (PersonaFinish::Muted, None) => hex,
    }
}

/// One layer: an SVG on the 100 x 100 canvas, stacked over the others (`data-part`). Each part
/// that moves is its own layer, because the stylesheet cannot reach inside an SVG on Blitz.
fn layer(part: &'static str, marks: Vec<Mark>, finish: PersonaFinish) -> Element {
    let marks: Vec<Mark> = marks
        .into_iter()
        .map(|mark| mark.recoloured(|hex| finished(hex, finish)))
        .collect();
    if marks.is_empty() {
        return rsx! {};
    }
    rsx! {
        svg {
            class: "ds-persona-layer",
            "data-part": part,
            // `data-ds-svg` tells the markup lint this vector is quire's own, not raw SVG.
            "data-ds-svg": "persona",
            view_box: "0 0 100 100",
            "aria-hidden": "true",
            for mark in marks {
                {path(mark)}
            }
        }
    }
}

fn path(mark: Mark) -> Element {
    let (fill, stroke, width) = mark.attrs();
    rsx! {
        path {
            d: mark.d,
            fill,
            stroke,
            "stroke-width": width,
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            transform: mark.turn,
            opacity: mark.opacity,
        }
    }
}

/// The user's character at `size`, playing `mood` (design/24-PERSONA.md). Idle blinks and
/// breathes for 20 s after mounting, after each new `wake` stamp and after each mood change,
/// then holds still; each mood change plays its motion once. Under Reduced motion moods change
/// instantly and nothing blinks. `finish` mutes its colours with the icons' Muted style. Decorative: the name beside it is what a screen reader reads.
#[component]
pub fn Persona(
    spec: PersonaSpec,
    size: PersonaSize,
    #[props(default)] mood: Mood,
    #[props(default)] wake: WakeStamp,
    #[props(default)] finish: PersonaFinish,
) -> Element {
    let scheme = use_env().scheme;
    let life = life::use_life(spec, mood, wake);
    let detail = size.detail();
    let ground = finished(
        palette::ground(spec.backdrop, scheme, spec.tone).hex(),
        finish,
    );
    let style = format!(
        "--pa-ground:{ground};--pa-eyes:{}%",
        num(face::eye_line(&spec))
    );
    let (hop, hop_alias) = bump_attrs("ds-persona-hop", life.hop);
    let (wince, wince_alias) = bump_attrs("ds-persona-shake", life.wince);
    let (breathe, breathe_alias) = bump_attrs("ds-persona-breath", life.breathe);
    let (blink, blink_alias) = bump_attrs("ds-persona-blink", life.blink);
    let (drift, drift_alias) = bump_attrs("ds-persona-z", life.drift);
    rsx! {
        div {
            class: "ds-persona",
            "data-size": size.slug(),
            "data-mood": mood.slug(),
            "aria-hidden": "true",
            style,
            div { class: hop, "data-pulse": hop_alias,
                div { class: wince, "data-pulse": wince_alias,
                    div { class: breathe, "data-pulse": breathe_alias,
                        div { class: "ds-persona-tilt",
                            {layer("still", head::still(&spec, detail), finish)}
                            div { class: "ds-persona-look",
                                div { class: blink, "data-pulse": blink_alias,
                                    {layer("eyes", face::eyes(&spec, mood, detail), finish)}
                                }
                            }
                            {layer("brows", face::brows(&spec, mood, detail), finish)}
                            {layer("mouth", face::mouth(&spec, mood, detail), finish)}
                            {layer("worn", face::worn(&spec), finish)}
                        }
                    }
                }
            }
            if mood == Mood::Asleep {
                svg {
                    class: drift,
                    "data-pulse": drift_alias,
                    "data-ds-svg": "persona",
                    view_box: "0 0 100 100",
                    "aria-hidden": "true",
                    fill: "none",
                    stroke: "currentColor",
                    "stroke-width": "2.4",
                    "stroke-linecap": "round",
                    "stroke-linejoin": "round",
                    path { d: face::snooze(&spec) }
                }
            }
        }
    }
}
