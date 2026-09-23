//! Motion lab: fire each animation on a sample. Beside it, the CSS duration token at the level
//! the toolbar picked, the Rust `settle()` that times the state after it, and a live settle
//! timer that turns to "settled" when Rust believes the animation has ended.

use super::Section;
use super::motion::{millis, recipe_text};
use dioxus::prelude::*;
use ds::{
    Anim, Button, ButtonVariant, StaggerIndex, TimerPhase, settle, use_env, use_motion_timer,
    use_pulse,
};

/// The motion lab page.
#[component]
pub fn MotionLabPage() -> Element {
    let level = use_env().resolved.motion;
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
    let level = use_env().resolved.motion;
    let pulse = use_pulse(anim);
    let timer = use_motion_timer(anim);
    let (class, alias) = match pulse.attrs() {
        Some((class, alias)) => (format!("g-sample {class}"), Some(alias)),
        None => ("g-sample".to_string(), None),
    };
    let recipe = anim.recipe();
    let css = millis(recipe.duration.duration(level));
    let settles = millis(settle(anim, level, StaggerIndex::default()));
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
                    variant: ButtonVariant::Mini,
                    label: format!("{anim:?}"),
                    onclick: move |_| {
                        pulse.fire();
                        timer.start(EventHandler::new(|()| {}));
                    },
                }
                span { class: "g-code", "{recipe_text(anim)}" }
                span { class: "g-code", "css {recipe.duration.var().as_str()} = {css} · settle() = {settles} · {phase}" }
            }
        }
    }
}
