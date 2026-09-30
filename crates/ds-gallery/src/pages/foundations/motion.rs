//! Motion: every timing token at each of the four levels, and every animation's recipe with the
//! `settle()` that times the state after it.

use crate::pages::Section;
use dioxus::prelude::*;
use ds::Word;
use ds::{Anim, DelayToken, DurationToken, EasingToken, Fill, Iteration, MotionLevel, settle};
use std::time::Duration;

/// `170ms`, `5.2s`.
pub fn millis(duration: Duration) -> String {
    match duration.as_millis() {
        ms if ms >= 2000 && ms % 100 == 0 => format!("{}s", ms as f64 / 1000.0),
        ms => format!("{ms}ms"),
    }
}

/// The motion page.
#[component]
pub fn MotionPage() -> Element {
    rsx! {
        crate::pages::foundations::motion_driven::DrivenSection {}
        Section { title: "Durations", note: "Every --t token at Standard and Reduced (60 ms everywhere).",
            LevelTable {
                rows: DurationToken::ALL.iter().map(|token| {
                    (token.var().as_str().to_string(), MotionLevel::ALL.iter().copied().map(|level| millis(token.duration(level))).collect::<Vec<_>>())
                }).collect::<Vec<_>>(),
            }
        }
        Section { title: "Delays and holds", note: "Rust-timed delays, never scaled by the level.",
            LevelTable {
                rows: DelayToken::ALL.iter().map(|token| {
                    (format!("{token:?}"), MotionLevel::ALL.iter().map(|_| millis(token.delay())).collect::<Vec<_>>())
                }).collect::<Vec<_>>(),
            }
        }
        Section { title: "Easings", note: "No easing follows the level.",
            LevelTable {
                rows: EasingToken::ALL.iter().map(|token| {
                    (token.var().as_str().to_string(), MotionLevel::ALL.iter().copied().map(|level| token.easing(level).css()).collect::<Vec<_>>())
                }).collect::<Vec<_>>(),
            }
        }
        Section { title: "Animations", note: "Each Anim's keyframes, tokens, fill and iteration, and settle(anim, level, 0) at each level: when Rust takes the state after it.",
            div { class: "g-table g-cols6",
                span { class: "g-head", "anim" }
                span { class: "g-head", "recipe" }
                for level in MotionLevel::ALL.iter().copied() {
                    span { class: "g-head", "settle {level.slug()}" }
                }
                for anim in Anim::ALL {
                    span { class: "g-code", "{anim:?}" }
                    span { class: "g-code", "{recipe_text(anim)}" }
                    for level in MotionLevel::ALL.iter().copied() {
                        span { class: "g-code", "{millis(settle(anim, level))}" }
                    }
                }
            }
        }
    }
}

/// `fold var(--t-big) var(--e-exit) forwards`.
pub fn recipe_text(anim: Anim) -> String {
    let recipe = anim.recipe();
    let fill = match recipe.fill {
        Fill::None => "",
        Fill::Forwards => " forwards",
        Fill::Backwards => " backwards",
    };
    let iteration = match recipe.iteration {
        Iteration::Once => "",
        Iteration::Infinite => " infinite",
        Iteration::InfiniteAlternate => " infinite alternate",
    };
    format!(
        "{} {} {}{iteration}{fill}",
        recipe.keyframes,
        recipe.duration.var().as_str(),
        recipe.easing.var().as_str()
    )
}

/// One row per token, one column per level.
#[component]
fn LevelTable(rows: Vec<(String, Vec<String>)>) -> Element {
    rsx! {
        div { class: "g-table g-cols5",
            span { class: "g-head", "token" }
            for level in MotionLevel::ALL.iter().copied() {
                span { class: "g-head", "{level.slug()}" }
            }
            for (name , values) in rows {
                span { class: "g-code", "{name}" }
                for value in values {
                    span { class: "g-code", "{value}" }
                }
            }
        }
    }
}
