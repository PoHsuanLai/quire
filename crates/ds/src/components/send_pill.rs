//! SendPill: undo send, a countdown ring and an Undo, then "Sent" (design/04-COMPONENTS.md
//! section 31, design/06-INTERACTIONS.md section 10).
//!
//! The consumer owns the countdown (`SendCountdown` 5 s in `SendTick` 1 s steps) and passes the
//! elapsed share as `progress`; the ring drains by it, drawn as SVG attributes because a CSS
//! transition on `stroke-dashoffset` does not run in Blitz (O-20, spike S6). The pill springs up
//! on the frame after it mounts. Undo is offered while counting; once done it hides Undo and
//! slides away after `SentHold` (1600 ms).

use crate::components::vocab::Fraction;
use crate::root::env::Env;
use crate::time::{FRAME_SLACK, sleep};
use crate::tokens::DelayToken;
use dioxus::prelude::*;

/// The ring's circumference as S rounds it: `stroke-dasharray:57` for r = 9 (`S:713`).
const RING: u16 = 57;

/// Where the pill is: `data-shown`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shown {
    /// Below the card edge.
    Hidden,
    /// Up.
    Shown,
}

impl SendPhase {
    /// The `data-phase` word.
    fn slug(self) -> &'static str {
        match self {
            SendPhase::Counting => "counting",
            SendPhase::Done => "done",
        }
    }
}

/// The run circle's `stroke-dashoffset`: 0 at the start, the whole ring once `progress` is all
/// of it.
fn drained(progress: Fraction) -> String {
    let permille = u32::from(progress.clamped().0);
    let offset = u32::from(RING) * permille;
    let text = format!("{}.{:03}", offset / 1000, offset % 1000);
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// Where the send is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SendPhase {
    /// Counting down; Undo is offered.
    Counting,
    /// Sent; Undo is gone.
    Done,
}

/// The undo-send pill.
#[component]
pub fn SendPill(
    text: String,
    progress: Fraction,
    phase: SendPhase,
    onundo: EventHandler<()>,
) -> Element {
    let mut shown = use_signal(|| Shown::Hidden);
    use_hook(move || {
        spawn(async move {
            sleep(FRAME_SLACK).await;
            shown.set(Shown::Shown);
        })
    });
    // Done: "Sent" stays up for SentHold, then the pill slides away (S:2342).
    let env = use_hook(try_consume_context::<Signal<Env>>);
    let mut held = use_hook(|| CopyValue::new(SendPhase::Counting));
    if phase == SendPhase::Done && *held.peek() == SendPhase::Counting {
        held.set(SendPhase::Done);
        let level = env
            .map(|env| env.peek().resolved.motion)
            .unwrap_or_default();
        let hold = DelayToken::SentHold.delay(level);
        spawn(async move {
            sleep(hold).await;
            shown.set(Shown::Hidden);
        });
    }
    let shown = match shown() {
        Shown::Hidden => "hidden",
        Shown::Shown => "shown",
    };
    rsx! {
        div {
            class: "ds-send-pill",
            "data-shown": shown,
            "data-phase": phase.slug(),
            role: "status",
            style: "--f:{progress.css()}",
            // The circles carry no classes: CSS does not reach inside an SVG on Blitz (S6).
            svg {
                class: "ds-send-ring",
                view_box: "0 0 24 24",
                width: "20",
                height: "20",
                "aria-hidden": "true",
                circle {
                    cx: "12",
                    cy: "12",
                    r: "9",
                    fill: "none",
                    stroke: "currentColor",
                    "stroke-width": "3",
                    opacity: ".25",
                }
                circle {
                    cx: "12",
                    cy: "12",
                    r: "9",
                    fill: "none",
                    stroke: "currentColor",
                    "stroke-width": "3",
                    "stroke-linecap": "round",
                    "stroke-dasharray": "{RING}",
                    "stroke-dashoffset": drained(progress),
                    transform: "rotate(-90 12 12)",
                }
            }
            span { "{text}" }
            if phase == SendPhase::Counting {
                button {
                    r#type: "button",
                    class: "ds-send-pill-undo",
                    onclick: move |_| onundo.call(()),
                    "Undo"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::drained;
    use crate::components::vocab::Fraction;

    #[test]
    fn the_ring_drains_with_progress() {
        const CASES: &[(u16, &str)] = &[(0, "0"), (400, "22.8"), (1000, "57"), (1400, "57")];
        for &(permille, want) in CASES {
            assert_eq!(drained(Fraction(permille)), want, "{permille}");
        }
    }
}
