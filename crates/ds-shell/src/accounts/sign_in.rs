//! SignInForm: the fields a sign-in asks for (design/31 section 5.6, PLAN 2.6): the address, a
//! password, an app password or an API key, a server when the lookup missed it. The host owns
//! every value and hears each keystroke; Continue stays disabled until every required field has
//! text, and Return does the same as Continue. A refusal marks its field with a sentence under it
//! (a password shakes once for each new refused attempt).

use super::adapter::{Action, Entry, Intent, Landing, Rejection};
use super::frame::StepFrame;
use super::model::{FieldProblem, FieldRole, FieldText, FormField, StepTitle};
use super::wording::{field_label, field_placeholder, form_ready, problem_text};
use dioxus::prelude::*;
use ds::components::content::provider_mark::MarkProvider;
use ds_core::vocab::Availability;
use ds_core::word::Word;

/// The sign-in form for `provider`, whose round `mark` leads the header. `on_input` hears the role of the field and what it now
/// holds; secret text arrives as [`FieldText::Secret`] and nowhere else. `on_submit` is Continue
/// or Return, heard only while the form is ready.
#[component]
pub fn SignInForm(
    #[props(into)] provider: String,
    mark: MarkProvider,
    fields: Vec<FormField>,
    #[props(default)] problem: Option<FieldProblem>,
    on_input: EventHandler<(FieldRole, FieldText)>,
    on_submit: EventHandler<()>,
    on_back: EventHandler<()>,
    on_cancel: EventHandler<()>,
    #[props(default)] title: StepTitle,
) -> Element {
    let ready = form_ready(&fields);
    let availability = if ready {
        Availability::Enabled
    } else {
        Availability::Disabled
    };
    let enter = ready.then(|| EventHandler::new(move |()| on_submit.call(())));
    rsx! {
        StepFrame {
            shown: title,
            step: "sign-in",
            title: "Sign in to {provider}",
            mark,
            onenter: enter,
            oncancel: on_cancel,
            body: rsx! {
                for (index , field) in fields.iter().enumerate() {
                    {
                        let role = field.role;
                        let rejection = problem
                            .filter(|problem| problem.role == role)
                            .map(|problem| Rejection {
                                message: problem_text(problem),
                                attempt: problem.attempt.0,
                            });
                        rsx! {
                            Entry {
                                key: "{role.slug()}",
                                label: field_label(role),
                                text: field.text.clone(),
                                placeholder: field_placeholder(role),
                                landing: if index == 0 { Landing::Here } else { Landing::Anywhere },
                                rejection,
                                oninput: move |text: FieldText| on_input.call((role, text)),
                            }
                        }
                    }
                }
            },
            actions: rsx! {
                Action { label: "Back", onclick: move |()| on_back.call(()) }
                Action { label: "Cancel", intent: Intent::Cancel, onclick: move |()| on_cancel.call(()) }
                Action { label: "Continue", intent: Intent::Default, availability, onclick: move |()| on_submit.call(()) }
            },
        }
    }
}
