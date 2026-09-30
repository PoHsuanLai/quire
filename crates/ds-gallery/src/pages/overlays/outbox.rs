//! The send pill's outbox states: Cancel for a held send, the failed moods on demand, and a
//! refusal under the text. Beside the live countdown on
//! the pills page.

use crate::pages::Specimen;
use dioxus::prelude::*;
use ds::components::app::send_mood::SendMood;
use ds::components::app::send_pill::{PillAction, SendPill};
use ds::detail::{Operation, PendingToken};
use ds::prelude::*;
use ds_style::tokens::control_size::ControlSize;

/// The moods a person can play here, with the words the pill says in each.
const MOODS: [(SendMood, &str, &str); 2] = [
    (SendMood::Calm, "Calm", "Waiting in the outbox"),
    (SendMood::Fatal, "Fatal", "Not sent"),
];

/// Three stages: a held send, a live mood picker, and a refused take-back.
#[component]
pub fn Outbox() -> Element {
    let mut mood = use_signal(|| SendMood::Calm);
    let text = MOODS
        .iter()
        .find(|(each, _, _)| *each == mood())
        .map_or("", |(_, _, text)| *text);
    rsx! {
        Specimen { name: "scheduled: Cancel",
            div { class: "g-stage",
                SendPill { text: "Scheduled for 9:00", progress: Fraction(0), operation: Operation::Running(PendingToken::start()), action: PillAction::Cancel, onundo: |_| {} }
            }
        }
        Specimen { name: "failed: pick a mood",
            div { class: "g-stage-pad",
                for (each , name , _) in MOODS {
                    Button { size: ControlSize::Mini, label: name, onclick: move |_| mood.set(each) }
                }
            }
            div { class: "g-stage",
                SendPill { text, progress: Fraction(700), operation: Operation::Running(PendingToken::start()), mood: mood(), action: PillAction::Nothing, onundo: |_| {} }
            }
        }
        Specimen { name: "refused",
            div { class: "g-stage",
                SendPill { text: "Not sent", progress: Fraction(700), operation: Operation::Running(PendingToken::start()), mood: SendMood::Fatal, refusal: "No recipients", onundo: |_| {} }
            }
        }
    }
}
