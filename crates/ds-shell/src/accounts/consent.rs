//! ConsentAlert: an app asks to use an account (design/31 section 4.5). It stands in a narrow
//! centred sheet as an alert does: the app's icon, "Photos wants to keep its library in your
//! files", the account it would use (a pop-up when several fit), and Allow Once, Always Allow and
//! Don't Allow stacked (This Session Only between the first two when the host offers it), the first the default. Escape closes the sheet and answers
//! [`ConsentAnswer::Dismiss`], which stores nothing; only "Don't Allow" denies. With no account
//! that fits it offers "Add Account..." instead.

use super::adapter::{Action, Intent, Landing};
use super::model::{
    AccountChoice, AllowScope, ChoiceKey, ConsentAnswer, PickerChoice, SessionOffer,
};
use super::picker::AccountPicker;
use super::wording::{allow_label, consent_message, consent_title, effective_choice};
use dioxus::prelude::*;
use ds::components::content::icon_source::IconSource;
use ds::components::content::icon_view::IconView;
use ds::components::overlays::sheet::Sheet;
use ds::components::overlays::sheet_attach::Attach;
use ds::components::overlays::sheet_width::SheetWidth;
use ds_style::icon::render::IconSize;

/// The consent column: the app's icon, the words, the account pop-up and the buttons, bare, so a
/// host puts it in a window of its own. Return allows once (or adds an account when none fits),
/// Escape answers `Dismiss`. `ConsentAlert` is this in a narrow centred sheet.
///  `request` finishes "{app} wants to ..."; `choices` are the accounts that
/// fit, `chosen` the one the host has picked (the first when it has picked none, or none that
/// fits). `session` says whether "This Session Only" is offered. `on_choose` hears a pick in the pop-up; `on_answer` the person's answer.
#[component]
pub fn ConsentBody(
    #[props(into)] app: String,
    #[props(into)] request: String,
    choices: Vec<AccountChoice>,
    #[props(default)] chosen: Option<ChoiceKey>,
    #[props(default)] icon: Option<IconSource>,
    #[props(default)] session: SessionOffer,
    on_choose: EventHandler<ChoiceKey>,
    on_answer: EventHandler<ConsentAnswer>,
) -> Element {
    let title = consent_title(&app, &request);
    let account = effective_choice(&choices, chosen.as_ref()).map(|choice| choice.key.clone());
    let message = consent_message(&choices);
    let answer = move |scope: AllowScope| {
        let account = account.clone();
        move |()| {
            if let Some(account) = account.clone() {
                on_answer.call(ConsentAnswer::Allow { account, scope });
            }
        }
    };
    let once = EventHandler::new(answer(AllowScope::Once));
    let this_session = EventHandler::new(answer(AllowScope::Session));
    let always = EventHandler::new(answer(AllowScope::Always));
    let picked = effective_choice(&choices, chosen.as_ref()).map(|choice| choice.key.clone());
    let several = choices.len() > 1;
    let fits = !choices.is_empty();
    rsx! {
        div {
            class: "ds-alert",
            onkeydown: move |event| match event.key() {
                Key::Escape => {
                    event.stop_propagation();
                    event.prevent_default();
                    on_answer.call(ConsentAnswer::Dismiss);
                }
                Key::Enter => {
                    event.stop_propagation();
                    event.prevent_default();
                    if fits { once.call(()) } else { on_answer.call(ConsentAnswer::AddAccount) }
                }
                _ => {}
            },
            if let Some(icon) = icon {
                div { class: "ds-alert-icon", IconView { source: icon, size: IconSize::Tile48 } }
            }
            div { class: "ds-alert-title", "{title}" }
            div { class: "ds-alert-body", "{message}" }
            if several {
                div { class: "ds-acc-choice",
                    AccountPicker {
                        accounts: choices.clone(),
                        chosen: picked,
                        on_pick: move |choice: PickerChoice| match choice {
                            PickerChoice::Account(key) => on_choose.call(key),
                            PickerChoice::Add => on_answer.call(ConsentAnswer::AddAccount),
                        },
                    }
                }
            }
            div { class: "ds-alert-footer", "data-layout": if fits { "stack" } else { "row" },
                if fits {
                    span { class: "ds-alert-slot",
                        Action { label: allow_label(AllowScope::Once), intent: Intent::Default, landing: Landing::Here, onclick: move |()| once.call(()) }
                    }
                    if session == SessionOffer::Offered {
                        span { class: "ds-alert-slot",
                            Action { label: allow_label(AllowScope::Session), onclick: move |()| this_session.call(()) }
                        }
                    }
                    span { class: "ds-alert-slot",
                        Action { label: allow_label(AllowScope::Always), onclick: move |()| always.call(()) }
                    }
                    span { class: "ds-alert-slot",
                        Action { label: "Don't Allow", onclick: move |()| on_answer.call(ConsentAnswer::Deny) }
                    }
                } else {
                    span { class: "ds-alert-slot",
                        Action { label: "Add Account\u{2026}", intent: Intent::Default, landing: Landing::Here, onclick: move |()| on_answer.call(ConsentAnswer::AddAccount) }
                    }
                    span { class: "ds-alert-slot",
                        Action { label: "Cancel", intent: Intent::Cancel, onclick: move |()| on_answer.call(ConsentAnswer::Dismiss) }
                    }
                }
            }
        }

    }
}

/// The consent alert in a narrow centred sheet: [`ConsentBody`] for a host with no window of its
/// own. Escape closes the sheet and answers `Dismiss`.
#[component]
pub fn ConsentAlert(
    #[props(into)] app: String,
    #[props(into)] request: String,
    choices: Vec<AccountChoice>,
    #[props(default)] chosen: Option<ChoiceKey>,
    #[props(default)] icon: Option<IconSource>,
    #[props(default)] session: SessionOffer,
    on_choose: EventHandler<ChoiceKey>,
    on_answer: EventHandler<ConsentAnswer>,
) -> Element {
    let label = consent_title(&app, &request);
    rsx! {
        Sheet {
            label,
            onclose: move |()| on_answer.call(ConsentAnswer::Dismiss),
            attach: Attach::Centre,
            width: SheetWidth::Narrow,
            ConsentBody { app, request, choices, chosen, icon, session, on_choose, on_answer }
        }
    }
}
