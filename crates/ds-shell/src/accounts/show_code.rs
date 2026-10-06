//! ShowCode: the device-code sign-in (PLAN 2.6): the code the person types at the provider's page,
//! large, the address of that page, and "Copy Code". Used where no browser can be opened for the
//! person (a headless or remote session).

use super::adapter::{Action, CopyAction, Intent, Landing};
use super::frame::StepFrame;
use super::model::CopyState;
use dioxus::prelude::*;

/// The device-code step for `provider`: `code` to type at `url`. `copied` is the host's word on
/// whether the code has been copied; `on_copy` hears the code to copy.
#[component]
pub fn ShowCode(
    #[props(into)] provider: String,
    code: String,
    url: String,
    #[props(default)] copied: CopyState,
    on_copy: EventHandler<String>,
    on_cancel: EventHandler<()>,
) -> Element {
    let copy = code.clone();
    rsx! {
        StepFrame {
            step: "code",
            title: "Enter this code",
            onenter: EventHandler::new(move |()| on_copy.call(copy.clone())),
            oncancel: on_cancel,
            body: rsx! {
                div { class: "ds-acc-code", role: "img", "aria-label": "Code {code}", "{code}" }
                div { class: "ds-acc-caption", "Go to this page on any device and enter the code to sign in to {provider}." }
                div { class: "ds-acc-url", "{url}" }
            },
            actions: rsx! {
                Action { label: "Cancel", intent: Intent::Cancel, onclick: move |()| on_cancel.call(()) }
                CopyAction { label: "Copy Code", copied: "Copied", text: code.clone(), state: copied, intent: Intent::Default, landing: Landing::Here, oncopy: on_copy }
            },
        }
    }
}
