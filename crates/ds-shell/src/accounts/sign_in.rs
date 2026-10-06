//! SignInForm: the fields a sign-in asks for (design/31 section 5.6, PLAN 2.6): the address, a
//! password, an app password or an API key, a server when the lookup missed it. The host owns
//! every value and hears each keystroke; Continue stays disabled until every required field has
//! text, and Return does the same as Continue. A refusal marks its field with a sentence under it
//! (a password shakes once for each new refused attempt).

use super::adapter::{Action, Entry, Intent, Landing, Pick, Rejection};
use super::frame::StepFrame;
use super::model::{FieldProblem, FieldRole, FieldText, FormField, FormPart, StepTitle};
use super::wording::{field_label, field_placeholder, form_ready, part_words, problem_text};
use dioxus::prelude::*;
use ds::components::content::provider_mark::MarkProvider;
use ds::components::fields::field_row::{FieldRow, RowLayout};
use ds::components::forms::{form::Form, form_section::FormSection};
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
    let ready = form_ready(&fields, problem);
    let grouped = fields
        .iter()
        .any(|field| field.choices.is_some() || field.part.is_some());
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
                if grouped {
                    Form {
                        for (part , run) in runs(&fields) {
                            FormSection { key: "{part.map_or(\"form\", |part| part.slug())}", title: part.map(part_title),
                                for (index , field) in run {
                                    {row(field, index == 0, problem, on_input)}
                                }
                            }
                        }
                    }
                } else {
                    for (index , field) in fields.iter().enumerate() {
                        {entry(field, index == 0, problem, on_input)}
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

/// The part a run of fields sits in, and the fields of the run with their place in the form.
type Run<'a> = (Option<FormPart>, Vec<(usize, &'a FormField)>);

/// The fields cut into runs of neighbours that share a part, in the order given.
fn runs(fields: &[FormField]) -> Vec<Run<'_>> {
    fields
        .iter()
        .enumerate()
        .fold(Vec::<Run>::new(), |mut runs, (index, field)| {
            match runs.last_mut() {
                Some((part, run)) if *part == field.part => run.push((index, field)),
                _ => runs.push((field.part, vec![(index, field)])),
            }
            runs
        })
}

/// What the person's eye must not need to guess: the words over a group.
fn part_title(part: FormPart) -> String {
    part_words(part).to_owned()
}

/// The mark under `problem` for the field of `role`, if it is that one.
fn rejection(problem: Option<FieldProblem>, role: FieldRole) -> Option<Rejection> {
    problem
        .filter(|problem| problem.role == role)
        .map(|problem| Rejection {
            message: problem_text(problem),
            attempt: problem.attempt.0,
        })
}

/// A text entry for `field`: its hint when the host gave one, else the role's.
fn entry(
    field: &FormField,
    first: bool,
    problem: Option<FieldProblem>,
    on_input: EventHandler<(FieldRole, FieldText)>,
) -> Element {
    let role = field.role;
    rsx! {
        Entry {
            key: "{role.slug()}",
            label: field_label(role),
            text: field.text.clone(),
            placeholder: field
                .hint
                .clone()
                .unwrap_or_else(|| field_placeholder(role, field.requirement).to_owned()),
            landing: if first { Landing::Here } else { Landing::Anywhere },
            rejection: rejection(problem, role),
            oninput: move |text: FieldText| on_input.call((role, text)),
        }
    }
}

/// One row of a grouped form: the label, and a pop-up (a choice) or an entry beside it.
fn row(
    field: &FormField,
    first: bool,
    problem: Option<FieldProblem>,
    on_input: EventHandler<(FieldRole, FieldText)>,
) -> Element {
    let role = field.role;
    let control = match &field.choices {
        Some(choices) => {
            let chosen = match &field.text {
                FieldText::Plain(slug) => Some(slug.clone()),
                FieldText::Secret(_) => None,
            };
            rsx! {
                Pick {
                    label: field_label(role),
                    choices: choices.clone(),
                    chosen,
                    onpick: move |slug: String| on_input.call((role, FieldText::Plain(slug))),
                }
            }
        }
        None => entry(field, first, problem, on_input),
    };
    rsx! {
        FieldRow { key: "{role.slug()}", label: field_label(role), layout: RowLayout::Form, {control} }
    }
}
