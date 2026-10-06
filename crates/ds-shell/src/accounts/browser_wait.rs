//! BrowserWait: the person finishes signing in in their browser (design/31 section 5.6): the
//! address that was opened, "Copy Link" for a browser the desktop did not pick, "Open Again" and
//! Cancel. Return opens the page again.

use super::adapter::{Action, CopyAction, Intent, Landing};
use super::frame::StepFrame;
use super::model::CopyState;
use super::waiting::Waiting;
use dioxus::prelude::*;

/// The browser step for `provider`, which opened `url`. `copied` is the host's word on whether the
/// link has been copied; `on_copy` hears the link to copy.
#[component]
pub fn BrowserWait(
    #[props(into)] provider: String,
    url: String,
    #[props(default)] copied: CopyState,
    on_open_again: EventHandler<()>,
    on_copy: EventHandler<String>,
    on_cancel: EventHandler<()>,
) -> Element {
    rsx! {
        StepFrame {
            step: "browser",
            title: "Continue in your browser",
            onenter: EventHandler::new(move |()| on_open_again.call(())),
            oncancel: on_cancel,
            body: rsx! {
                div { class: "ds-acc-waiting",
                    Waiting {}
                    span { "Finish signing in to {provider} in the window that opened." }
                }
                div { class: "ds-acc-url", "{url}" }
            },
            actions: rsx! {
                CopyAction { label: "Copy Link", copied: "Copied", text: url.clone(), state: copied, oncopy: on_copy }
                Action { label: "Cancel", intent: Intent::Cancel, onclick: move |()| on_cancel.call(()) }
                Action { label: "Open Again", intent: Intent::Default, landing: Landing::Here, onclick: move |()| on_open_again.call(()) }
            },
        }
    }
}
