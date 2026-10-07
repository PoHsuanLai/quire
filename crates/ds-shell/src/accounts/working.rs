//! SignInWorking: something is running (a lookup, the sign-in, the store) and nothing but Cancel
//! can be pressed.

use super::adapter::{Action, Intent, Landing};
use super::frame::StepFrame;
use super::model::StepTitle;
use super::waiting::Waiting;
use dioxus::prelude::*;
use ds::components::content::provider_mark::{MarkProvider, MarkStyle};

/// What the working step is waiting on.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum WorkingFor {
    /// The provider's own sign-in.
    #[default]
    Provider,
    /// An agent login, run by the agent called this.
    Agent(String),
}

impl WorkingFor {
    /// The line under the title.
    fn line(&self) -> String {
        match self {
            Self::Provider => "This takes a moment.".to_owned(),
            Self::Agent(label) => format!("Waiting for {label}\u{2026}"),
        }
    }
}

/// The waiting step for `provider`. Escape and Cancel call `on_cancel`; Return does nothing.
#[component]
pub fn SignInWorking(
    #[props(into)] provider: String,
    on_cancel: EventHandler<()>,
    #[props(default)] title: StepTitle,
    #[props(default)] mark: Option<MarkProvider>,
    #[props(default)] style: MarkStyle,
    #[props(default)] waiting_for: WorkingFor,
) -> Element {
    let line = waiting_for.line();
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
                    span { "{line}" }
                }
            },
            actions: rsx! {
                Action { label: "Cancel", intent: Intent::Cancel, landing: Landing::Here, onclick: move |()| on_cancel.call(()) }
            },
        }
    }
}
