//! SignInFailed: the sign-in ended without an account, and why. The person tries again (when a
//! retry could help), goes back a step, or is done. A fault may carry one more action of its own.

use super::adapter::{Action, Intent, Landing};
use super::frame::StepFrame;
use super::model::SignInFault;
use super::model::StepTitle;
use super::wording::fault;
use dioxus::prelude::*;
use ds::components::content::provider_mark::MarkProvider;

/// Whether trying again could help, as the host knows from the fault.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Recovery {
    /// A retry may work: "Try Again" is the default button.
    #[default]
    Retry,
    /// Only a change outside the sheet helps: there is no "Try Again", and "Done" is the default.
    NoRetry,
}

impl SignInFault {
    /// Whether a retry can help after this fault: only `AlreadyAdded` says no, since the
    /// account stays added however often the person tries. `SignInFailed` uses it when its
    /// `recovery` is left out.
    pub fn recovery(self) -> Recovery {
        match self {
            SignInFault::AlreadyAdded => Recovery::NoRetry,
            SignInFault::Refused
            | SignInFault::Unreachable
            | SignInFault::Unreadable
            | SignInFault::NeedsClientId
            | SignInFault::TimedOut
            | SignInFault::Cancelled
            | SignInFault::Forbidden
            | SignInFault::StoreFailed
            | SignInFault::NoLauncher
            | SignInFault::NotInstalled
            | SignInFault::NotRunning
            | SignInFault::SignedOut
            | SignInFault::NotAllowed => Recovery::Retry,
        }
    }
}

/// One more button for a fault, drawn before the default: "Open Settings", "Learn More". The
/// host supplies its words and what it does.
#[derive(Debug, Clone, PartialEq)]
pub struct FailureAction {
    /// The button's words.
    pub label: String,
    /// Called when the person presses it.
    pub on_press: EventHandler<()>,
}

/// The failure step for `provider`. With `Recovery::Retry` (the default, unless the fault says otherwise) Return is "Try Again"
/// and Cancel calls `on_cancel`; with `Recovery::NoRetry` (the default for [`SignInFault::AlreadyAdded`], see [`SignInFault::recovery`]) there is no "Try Again" (`on_retry` is
/// never called, so a host may leave it out) and Return is "Done", which calls `on_cancel`.
/// Escape calls `on_cancel` either way. `extra` adds one button of the host's.
#[component]
pub fn SignInFailed(
    #[props(into)] provider: String,
    why: SignInFault,
    #[props(default, into)] recovery: Option<Recovery>,
    #[props(default)] on_retry: EventHandler<()>,
    on_back: EventHandler<()>,
    on_cancel: EventHandler<()>,
    #[props(default)] extra: Option<FailureAction>,
    #[props(default)] title: StepTitle,
    #[props(default)] mark: Option<MarkProvider>,
) -> Element {
    let sentence = fault(why, &provider, mark);
    let recovery = recovery.unwrap_or_else(|| why.recovery());
    let enter = match recovery {
        Recovery::Retry => on_retry,
        Recovery::NoRetry => on_cancel,
    };
    rsx! {
        StepFrame {
            shown: title,
            step: "failed",
            title: "Could not add {provider}",
            onenter: EventHandler::new(move |()| enter.call(())),
            oncancel: on_cancel,
            body: rsx! {
                div { class: "ds-acc-caption", role: "alert", "{sentence}" }
            },
            actions: rsx! {
                Action { label: "Back", onclick: move |()| on_back.call(()) }
                match recovery {
                    Recovery::Retry => rsx! {
                        Action { label: "Cancel", intent: Intent::Cancel, onclick: move |()| on_cancel.call(()) }
                    },
                    Recovery::NoRetry => rsx! {},
                }
                if let Some(action) = extra {
                    Action { label: action.label, onclick: move |()| action.on_press.call(()) }
                }
                match recovery {
                    Recovery::Retry => rsx! {
                        Action { label: "Try Again", intent: Intent::Default, landing: Landing::Here, onclick: move |()| on_retry.call(()) }
                    },
                    Recovery::NoRetry => rsx! {
                        Action { label: "Done", intent: Intent::Default, landing: Landing::Here, onclick: move |()| on_cancel.call(()) }
                    },
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ds_core::word::Word;

    #[test]
    fn only_an_account_already_added_needs_no_retry_by_default() {
        for fault in SignInFault::ALL {
            let want = match fault {
                SignInFault::AlreadyAdded => Recovery::NoRetry,
                _ => Recovery::Retry,
            };
            assert_eq!(fault.recovery(), want, "{fault:?}");
        }
    }
}
