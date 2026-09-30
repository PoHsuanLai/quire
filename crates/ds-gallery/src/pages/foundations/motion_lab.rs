//! Motion lab: fire each animation on a sample. Beside it, the CSS duration token at the level
//! the toolbar picked, the Rust `settle()` that times the state after it, and a live settle
//! timer that turns to "settled" when Rust believes the animation has ended.

use crate::pages::Section;
use crate::pages::foundations::motion::{millis, recipe_text};
use dioxus::prelude::*;
use ds::prelude::*;
use ds_motion::pulse_key::PulseKey;
use ds_motion::timer::TimerPhase;
use ds_style::tokens::control_size::ControlSize;

/// The motion lab page.
#[component]
pub fn MotionLabPage() -> Element {
    let level = use_scope().resolved.motion;
    rsx! {
        Section {
            title: format!("Every animation at {}", level.slug()),
            note: "Change the level in the toolbar and fire again: the CSS duration and settle() move together, because both come from one table.",
            div { class: "g-lab",
                for anim in Anim::ALL {
                    LabCell { key: "{anim:?}", anim }
                }
            }
        }
    }
}

/// One animation: its sample, its button, its numbers.
#[component]
fn LabCell(anim: Anim) -> Element {
    let level = use_scope().resolved.motion;
    let mut pulse = use_signal(|| PulseKey::rest(anim));
    let timer = use_motion_timer(anim);
    let (class, alias) = match pulse().attrs() {
        Some((class, alias)) => (format!("g-sample {class}"), Some(alias)),
        None => ("g-sample".to_string(), None),
    };
    let recipe = anim.recipe();
    let css = millis(recipe.duration.duration(level));
    let settles = millis(settle(anim, level));
    let phase = match timer.phase() {
        TimerPhase::Idle => "idle",
        TimerPhase::Running => "running",
        TimerPhase::Settled => "settled",
    };
    rsx! {
        div { class: "g-lab-cell",
            div { class, "data-pulse": alias, "Aa" }
            div { class: "g-col",
                Button {
                    size: ControlSize::Mini,
                    label: format!("{anim:?}"),
                    onclick: move |_| {
                        pulse.set(pulse().fired());
                        timer.start(EventHandler::new(|()| {}));
                    },
                }
                span { class: "g-code", "{recipe_text(anim)}" }
                span { class: "g-code", "css {recipe.duration.var().as_str()} = {css} · settle() = {settles} · {phase}" }
            }
        }
    }
}
