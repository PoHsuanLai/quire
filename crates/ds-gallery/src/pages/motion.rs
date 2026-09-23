//! Motion: every timing token at each of the four levels, and every animation's recipe with the
//! `settle()` that times the state after it.

use super::Section;
use dioxus::prelude::*;
use ds::{
    Anim, DelayToken, DurationToken, EasingToken, Fill, Iteration, MotionLevel, ScalarToken,
    StaggerIndex, settle,
};
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
        Section { title: "Durations", note: "Every --t token at Calm, Standard, Extra and Reduced (60 ms everywhere).",
            LevelTable {
                rows: DurationToken::ALL.iter().map(|token| {
                    (token.var().as_str().to_string(), MotionLevel::ALL.map(|level| millis(token.duration(level))))
                }).collect::<Vec<_>>(),
            }
        }
        Section { title: "Delays and holds", note: "Rust-timed delays; the two the stylesheet also reads carry their --d name.",
            LevelTable {
                rows: DelayToken::ALL.iter().map(|token| {
                    let name = token.var().map_or(format!("{token:?}"), |var| var.as_str().to_string());
                    (name, MotionLevel::ALL.map(|level| millis(token.delay(level))))
                }).collect::<Vec<_>>(),
            }
        }
        Section { title: "Easings", note: "Only the spring follows the level.",
            LevelTable {
                rows: EasingToken::ALL.iter().map(|token| {
                    (token.var().as_str().to_string(), MotionLevel::ALL.map(|level| token.easing(level).css()))
                }).collect::<Vec<_>>(),
            }
        }
        Section { title: "Scalars",
            LevelTable {
                rows: ScalarToken::ALL.iter().map(|token| {
                    (token.var().as_str().to_string(), MotionLevel::ALL.map(|level| token.value(level).css()))
                }).collect::<Vec<_>>(),
            }
        }
        Section { title: "Animations", note: "Each Anim's keyframes, tokens, fill and iteration, and settle(anim, level, 0) at each level: when Rust takes the state after it.",
            div { class: "g-table g-cols6",
                span { class: "g-head", "anim" }
                span { class: "g-head", "recipe" }
                for level in MotionLevel::ALL {
                    span { class: "g-head", "settle {level.slug()}" }
                }
                for anim in Anim::ALL {
                    span { class: "g-code", "{anim:?}" }
                    span { class: "g-code", "{recipe_text(anim)}" }
                    for level in MotionLevel::ALL {
                        span { class: "g-code", "{millis(settle(anim, level, StaggerIndex::default()))}" }
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
fn LevelTable(rows: Vec<(String, [String; 4])>) -> Element {
    rsx! {
        div { class: "g-table g-cols5",
            span { class: "g-head", "token" }
            for level in MotionLevel::ALL {
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
