//! The send pill's outbox states (mailo gaps 2): Cancel for a held send, the busy ring, the
//! failed moods played on demand, and a refusal under the text. Beside the live countdown on
//! the pills page.

use super::Specimen;
use dioxus::prelude::*;
use ds::{Button, ButtonVariant, Fraction, PillAction, SendMood, SendPhase, SendPill, SendRing};

/// The moods a person can play here, with the words the pill says in each.
const MOODS: [(SendMood, &str, &str); 4] = [
    (SendMood::Calm, "Calm", "Waiting in the outbox"),
    (SendMood::Nudge, "Nudge", "Not sent yet · will try again"),
    (SendMood::Shake, "Shake", "Not sent · sign in again"),
    (SendMood::Fatal, "Fatal", "Not sent"),
];

/// Four stages: a held send, a busy ring, a live mood picker, and a refused take-back.
#[component]
pub fn Outbox() -> Element {
    let mut mood = use_signal(|| SendMood::Calm);
    let text = MOODS
        .iter()
        .find(|(each, _, _)| *each == mood())
        .map_or("", |(_, _, text)| *text);
    let ring = match mood() {
        SendMood::Calm => SendRing::Spin,
        SendMood::Nudge | SendMood::Shake | SendMood::Fatal => SendRing::Drain,
    };
    rsx! {
        Specimen { name: "scheduled: Cancel",
            div { class: "g-stage",
                SendPill { text: "Scheduled for 9:00", progress: Fraction(0), phase: SendPhase::Counting, action: PillAction::Cancel, onundo: |_| {} }
            }
        }
        Specimen { name: "sending: the ring spins",
            div { class: "g-stage",
                SendPill { text: "Sending…", progress: Fraction(0), phase: SendPhase::Counting, ring: SendRing::Spin, action: PillAction::Nothing, onundo: |_| {} }
            }
        }
        Specimen { name: "failed: play a mood",
            div { class: "g-stage-pad",
                for (each , name , _) in MOODS {
                    Button { variant: ButtonVariant::Mini, label: name, onclick: move |_| mood.set(each) }
                }
            }
            div { class: "g-stage",
                SendPill { text, progress: Fraction(700), phase: SendPhase::Counting, mood: mood(), ring, action: PillAction::Nothing, onundo: |_| {} }
            }
        }
        Specimen { name: "refused",
            div { class: "g-stage",
                SendPill { text: "Not sent", progress: Fraction(700), phase: SendPhase::Counting, mood: SendMood::Fatal, refusal: "No recipients", onundo: |_| {} }
            }
        }
    }
}
