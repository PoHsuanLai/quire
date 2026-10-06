//! SignInWorking: something is running (a lookup, the sign-in, the store) and nothing but Cancel
//! can be pressed.

use super::adapter::{Action, Intent, Landing};
use super::frame::StepFrame;
use super::model::StepTitle;
use super::waiting::Waiting;
use dioxus::prelude::*;

/// The waiting step for `provider`. Escape and Cancel call `on_cancel`; Return does nothing.
#[component]
pub fn SignInWorking(
    #[props(into)] provider: String,
    on_cancel: EventHandler<()>,
    #[props(default)] title: StepTitle,
) -> Element {
    rsx! {
        StepFrame {
            shown: title,
            step: "working",
            title: "Signing in to {provider}",
            oncancel: on_cancel,
            body: rsx! {
                div { class: "ds-acc-waiting",
                    Waiting {}
                    span { "This takes a moment." }
                }
            },
            actions: rsx! {
                Action { label: "Cancel", intent: Intent::Cancel, landing: Landing::Here, onclick: move |()| on_cancel.call(()) }
            },
        }
    }
}
