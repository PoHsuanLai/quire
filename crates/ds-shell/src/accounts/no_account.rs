//! NoAccount: what an app shows when it has no account to use (design/31 section 5.1): an empty
//! state that says why, with the one action that can help.

use super::adapter::{Action, Intent};
use super::model::NoAccountWhy;
use super::wording::no_account;
use dioxus::prelude::*;
use ds::components::overlays::empty_state::EmptyState;
use ds_style::icon::Icon;

/// The empty state for `why`. "Add Account..." (`on_add`) for a missing account or a provider
/// that cannot do it; "Open Settings" (`on_settings`) for an app the person turned away.
#[component]
pub fn NoAccount(
    why: NoAccountWhy,
    on_add: EventHandler<()>,
    on_settings: EventHandler<()>,
) -> Element {
    let (title, line) = no_account(why);
    let action = match why {
        NoAccountWhy::NeedsAccount | NoAccountWhy::Unsupported => rsx! {
            Action { label: "Add Account\u{2026}", intent: Intent::Default, onclick: move |()| on_add.call(()) }
        },
        NoAccountWhy::Denied => rsx! {
            Action { label: "Open Settings", onclick: move |()| on_settings.call(()) }
        },
    };
    rsx! {
        EmptyState { title, description: Some(line.into()), icon: Some(Icon::Inbox), action: Some(action) }
    }
}
