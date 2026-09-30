//! SendPill: undo send, a countdown ring and an Undo, then "Sent" (design/30 section 2.11,
//! design/06-INTERACTIONS.md section 10).
//!
//! The consumer owns the countdown ([`SEND_COUNTDOWN`] in [`SEND_TICK`] steps) and passes the
//! elapsed share as `progress`; the ring is a `ProgressIndicator { style: Ring }` showing what is
//! left of it, so the arc drains as the share grows. The send is an [`Operation`]: `Running`
//! while it counts (Undo is offered), `Idle` once it is done (Undo goes, and after `ToastHold`
//! the pill slides away). The pill slides up on the frame after it mounts, by a CSS transition.
//!
//! C's outbox states ride on the same pill: a failed [`SendMood`] turns it `--danger`, the button
//! can say Cancel or be absent ([`PillAction`]), and a refusal reads as a second line under the
//! text.

use crate::components::app::send_mood::SendMood;
use crate::components::controls::progress::model::{Progress, ProgressStyle};
use crate::components::controls::progress::view::ProgressIndicator;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::time::{FRAME_SLACK, clock::sleep};
use ds_core::vocab::{Fraction, Shown};
use ds_core::word::Word;
use ds_motion::detail::operation::Operation;
use ds_style::tokens::control_size::ControlSize;
use ds_style::tokens::delay::DelayToken;
use std::time::Duration;

/// How long an undo-send countdown runs (proposed), the consumer's to count.
pub const SEND_COUNTDOWN: Duration = Duration::from_secs(5);

/// One step of that countdown (proposed).
pub const SEND_TICK: Duration = Duration::from_secs(1);

/// What is left of the countdown, for the ring: the whole ring at the start, none at the end.
fn remaining(progress: Fraction) -> Fraction {
    Fraction(1000 - progress.clamped().0)
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

/// Whether the send is still counting: `Running` counts, `Idle` is sent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sent {
    /// The countdown is running.
    Not,
    /// It went.
    Yes,
}

impl Sent {
    fn of(operation: Operation) -> Self {
        match operation {
            Operation::Running(_) => Sent::Not,
            Operation::Idle => Sent::Yes,
        }
    }
}

/// The undo-send pill.
///
/// `operation` is the send: `Running` while it counts down, `Idle` once it is sent. `Fatal` turns
/// the pill `--danger`. `action` is the button's word while counting (`onundo` hears it either
/// way). `refusal` is a second line under the text: why an Undo or Cancel was refused ("Too late
/// to take it back"), or "No recipients".
#[component]
pub fn SendPill(
    text: String,
    progress: Fraction,
    operation: Operation,
    onundo: EventHandler<()>,
    #[props(default)] mood: SendMood,
    #[props(default)] action: PillAction,
    #[props(default)] refusal: Option<String>,
    #[props(default)] common: Common,
) -> Element {
    let sent = Sent::of(operation);
    let word = match sent {
        Sent::Not => action.word(),
        Sent::Yes => None,
    };
    let mut shown = use_signal(|| Shown::Hidden);
    use_hook(move || {
        spawn(async move {
            sleep(FRAME_SLACK).await;
            shown.set(Shown::Visible);
        })
    });
    // Sent: "Sent" stays up for ToastHold, then the pill slides away (S:2342).
    let mut held = use_hook(|| CopyValue::new(Sent::Not));
    if sent == Sent::Yes && *held.peek() == Sent::Not {
        held.set(Sent::Yes);
        let hold = DelayToken::ToastHold.delay();
        spawn(async move {
            sleep(hold).await;
            shown.set(Shown::Hidden);
        });
    }
    let class = common.class("ds-send-pill");
    let data = common.data_attributes();
    let busy = (sent == Sent::Not).then_some("true");
    rsx! {
        div {
            id: common.id.clone(),
            class,
            "data-shown": shown().slug(),
            "data-mood": mood.attr(),
            role: "status",
            "aria-busy": busy,
            "aria-label": common.aria_label.clone(),
            onmounted: move |event| common.mounted(event),
            ..data,
            ProgressIndicator {
                style: ProgressStyle::Ring,
                progress: Progress::Known(remaining(progress)),
                size: ControlSize::Small,
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
    use super::remaining;
    use ds_core::vocab::Fraction;

    #[test]
    fn the_ring_holds_what_is_left_of_the_countdown() {
        const CASES: &[(u16, u16)] = &[(0, 1000), (400, 600), (1000, 0), (1400, 0)];
        for &(elapsed, want) in CASES {
            assert_eq!(remaining(Fraction(elapsed)), Fraction(want), "{elapsed}");
        }
    }
}
