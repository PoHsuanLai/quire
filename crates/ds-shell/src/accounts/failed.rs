//! SignInFailed: the sign-in ended without an account, and why. The person tries again, goes
//! back a step, or gives up.

use super::adapter::{Action, Intent, Landing};
use super::frame::StepFrame;
use super::model::SignInFault;
use super::wording::fault;
use dioxus::prelude::*;

/// The failure step for `provider`. Return is "Try Again", Escape and Cancel call `on_cancel`.
#[component]
pub fn SignInFailed(
    #[props(into)] provider: String,
    why: SignInFault,
    on_retry: EventHandler<()>,
    on_back: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let sentence = fault(why);
    rsx! {
        StepFrame {
            step: "failed",
            title: "Could not add {provider}",
            onenter: EventHandler::new(move |()| on_retry.call(())),
            oncancel: on_cancel,
            body: rsx! {
                div { class: "ds-acc-caption", role: "alert", "{sentence}" }
            },
            actions: rsx! {
                Action { label: "Back", onclick: move |()| on_back.call(()) }
                Action { label: "Cancel", intent: Intent::Cancel, onclick: move |()| on_cancel.call(()) }
                Action { label: "Try Again", intent: Intent::Default, landing: Landing::Here, onclick: move |()| on_retry.call(()) }
            },
        }
    }
}
