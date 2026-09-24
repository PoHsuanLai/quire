//! SendPill: undo send, a countdown ring and an Undo, then "Sent" (design/04-COMPONENTS.md
//! section 31, design/06-INTERACTIONS.md section 10).
//!
//! The consumer owns the countdown (`SendCountdown` 5 s in `SendTick` 1 s steps) and passes the
//! elapsed share as `progress`; the ring drains by it, drawn as SVG attributes because a CSS
//! transition on `stroke-dashoffset` does not run in Blitz (O-20, spike S6). The pill springs up
//! on the frame after it mounts. Undo is offered while counting; once done it hides Undo and
//! slides away after `SentHold` (1600 ms).
//!
//! C's outbox states ride on the same pill: a failed [`SendMood`] plays its one-shot, the ring
//! can spin while the send waits on the outbox ([`SendRing::Spin`]), the button can say Cancel
//! or be absent ([`PillAction`]), and a refusal reads as a second line under the text.

use crate::components::send_mood::{SendMood, use_mood_pulse};
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

/// What the pill's button offers while counting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PillAction {
    /// "Undo": take back a send in its grace period, or one waiting in the outbox.
    #[default]
    Undo,
    /// "Cancel": the same act, named for a send held for later.
    Cancel,
    /// No button: a send that is under way or has failed has nothing to take back.
    Nothing,
}

impl PillAction {
    /// The button's word, or `None` for no button.
    fn word(self) -> Option<&'static str> {
        match self {
            PillAction::Undo => Some("Undo"),
            PillAction::Cancel => Some("Cancel"),
            PillAction::Nothing => None,
        }
    }
}

/// What the ring does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SendRing {
    /// Drains by `progress`: the countdown, or held where it stopped.
    #[default]
    Drain,
    /// A short arc turning at the Spinner's `spin`: waiting on the outbox, no known end.
    Spin,
}

/// The run circle's dash for a spinning ring: an arc of 20 and a gap of the rest of 57.
const SPIN_DASH: &str = "20 37";

impl SendRing {
    /// `data-ring`: written only while spinning, so a draining pill's markup is as it was.
    fn slug(self) -> Option<&'static str> {
        match self {
            SendRing::Drain => None,
            SendRing::Spin => Some("spin"),
        }
    }
}

/// The run circle's `stroke-dasharray` and `stroke-dashoffset` for `ring` at `progress`.
fn run_dash(ring: SendRing, progress: Fraction) -> (String, String) {
    match ring {
        SendRing::Drain => (RING.to_string(), drained(progress)),
        SendRing::Spin => (SPIN_DASH.to_string(), "0".to_string()),
    }
}

/// The class list and `data-pulse` for a pill playing `pulse` (a failed mood's one-shot).
fn pulse_attrs(pulse: crate::components::vocab::PulseKey) -> (String, Option<&'static str>) {
    match pulse.attrs() {
        Some((anim, alias)) => (format!("ds-send-pill {anim}"), Some(alias)),
        None => ("ds-send-pill".to_string(), None),
    }
}

/// The undo-send pill.
///
/// `mood` plays its one-shot each time it changes to a failed mood (never on mount), and the
/// pill stays up; `Fatal` also turns it `--danger`. To play the same mood again, pass `Calm` for
/// a render first. `action` is the button's word while counting (`onundo` hears it either way).
/// `ring: Spin` turns a short arc instead of draining. `refusal` is a second line under the
/// text: why an Undo or Cancel was refused ("Too late to take it back"), or "No recipients".
#[component]
pub fn SendPill(
    text: String,
    progress: Fraction,
    phase: SendPhase,
    onundo: EventHandler<()>,
    #[props(default)] mood: SendMood,
    #[props(default)] action: PillAction,
    #[props(default)] ring: SendRing,
    #[props(default)] refusal: Option<String>,
) -> Element {
    let (class, alias) = pulse_attrs(use_mood_pulse(mood));
    let (dash, offset) = run_dash(ring, progress);
    let word = match phase {
        SendPhase::Counting => action.word(),
        SendPhase::Done => None,
    };
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
            class,
            "data-shown": shown,
            "data-phase": phase.slug(),
            "data-mood": mood.slug(),
            "data-pulse": alias,
            role: "status",
            style: "--f:{progress.css()}",
            // The circles carry no classes: CSS does not reach inside an SVG on Blitz (S6).
            // `data-ds-svg` tells the markup lint this vector is quire's own, not raw SVG.
            svg {
                class: "ds-send-ring",
                "data-ds-svg": "ring",
                "data-ring": ring.slug(),
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
                    "stroke-dasharray": dash,
                    "stroke-dashoffset": offset,
                    transform: "rotate(-90 12 12)",
                }
            }
            if let Some(why) = refusal {
                span { class: "ds-send-pill-text",
                    span { "{text}" }
                    span { class: "ds-send-pill-why", "{why}" }
                }
            } else {
                span { "{text}" }
            }
            if let Some(word) = word {
                button {
                    r#type: "button",
                    class: "ds-send-pill-undo",
                    onclick: move |_| onundo.call(()),
                    "{word}"
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
