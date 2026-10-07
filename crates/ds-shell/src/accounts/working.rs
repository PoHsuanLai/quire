//! SignInWorking: something is running (a lookup, the sign-in, the store) and nothing but Cancel
//! can be pressed.

use super::adapter::{Action, Intent, Landing};
use super::frame::StepFrame;
use super::model::StepTitle;
use super::waiting::Waiting;
use dioxus::prelude::*;
use ds::components::content::provider_mark::{MarkProvider, MarkStyle};

/// The waiting step for `provider`. Escape and Cancel call `on_cancel`; Return does nothing.
#[component]
pub fn SignInWorking(
    #[props(into)] provider: String,
    on_cancel: EventHandler<()>,
    #[props(default)] title: StepTitle,
    #[props(default)] mark: Option<MarkProvider>,
    #[props(default)] style: MarkStyle,
) -> Element {
    rsx! {
        StepFrame {
            shown: title,
            mark,
            style,
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
