//! The answer card component.

use super::action::{CardAction, CardActionId};
use super::actions::CardActions;
use super::draft_body::DraftBody;
use super::event_body::EventBody;
use super::footer::CardFooter;
use super::footer_view::CardFooterView;
use super::form_body::FormBody;
use super::model::AnswerView;
use super::plan_body::{PlanBody, plan_actions};
use super::refused_body::RefusedBody;
use super::replace_body::{ReplaceBody, replace_actions};
use super::text_body::TextBody;
use crate::components::companion::form::model::{FieldKeyId, FormValue};
use crate::root::common::Common;
use dioxus::prelude::*;

/// The `data-kind` word of a variant.
fn kind_of(view: &AnswerView) -> &'static str {
    match view {
        AnswerView::Text(_) => "text",
        AnswerView::DraftReply(_) => "draft-reply",
        AnswerView::ProposedEvent(_) => "proposed-event",
        AnswerView::Plan(_) => "plan",
        AnswerView::Replace(_) => "replace",
        AnswerView::Form(_) => "form",
        AnswerView::Refused(_) => "refused",
    }
}

/// The buttons a variant carries, in the order they are drawn.
fn actions_of(view: &AnswerView) -> Vec<CardAction> {
    match view {
        AnswerView::Text(answer) => answer.actions.clone(),
        AnswerView::DraftReply(draft) => draft.actions.clone(),
        AnswerView::ProposedEvent(event) => event.actions.clone(),
        AnswerView::Plan(plan) => plan_actions(plan),
        AnswerView::Replace(proposal) => replace_actions(proposal.phase),
        AnswerView::Form(form) => vec![form.submit.clone()],
        AnswerView::Refused(refused) => refused.actions.clone(),
    }
}

/// The footer every variant has.
fn footer_of(view: &AnswerView) -> CardFooter {
    match view {
        AnswerView::Text(answer) => answer.footer.clone(),
        AnswerView::DraftReply(draft) => draft.footer.clone(),
        AnswerView::ProposedEvent(event) => event.footer.clone(),
        AnswerView::Plan(plan) => plan.footer.clone(),
        AnswerView::Replace(proposal) => proposal.footer.clone(),
        AnswerView::Form(form) => form.footer.clone(),
        AnswerView::Refused(refused) => refused.footer.clone(),
    }
}

/// The variant's body.
#[component]
fn CardBody(view: AnswerView, on_edit: EventHandler<(FieldKeyId, FormValue)>) -> Element {
    match view {
        AnswerView::Text(answer) => rsx! { TextBody { answer } },
        AnswerView::DraftReply(draft) => rsx! { DraftBody { draft, on_edit } },
        AnswerView::ProposedEvent(event) => rsx! { EventBody { event } },
        AnswerView::Plan(view) => rsx! { PlanBody { view } },
        AnswerView::Replace(proposal) => rsx! { ReplaceBody { proposal } },
        AnswerView::Form(form) => {
            rsx! { FormBody { title: form.title, fields: form.fields, on_edit } }
        }
        AnswerView::Refused(refused) => rsx! { RefusedBody { why: refused.why } },
    }
}

/// An answer as a card: the variant's body, its actions and the footer every card has.
/// `focused` is the index of the action with the keyboard; `on_action` hears the action picked
/// and `on_edit` an edit in a draft or a form.
#[component]
pub fn AnswerCard(
    view: AnswerView,
    #[props(default)] focused: Option<usize>,
    on_action: EventHandler<CardActionId>,
    #[props(default)] on_edit: EventHandler<(FieldKeyId, FormValue)>,
    #[props(default)] common: Common,
) -> Element {
    let class = common.class("ds-answer-card");
    let data = common.data_attributes();
    let kind = kind_of(&view);
    let actions = actions_of(&view);
    let footer = footer_of(&view);
    rsx! {
        div {
            id: common.id.clone(),
            class,
            "data-kind": kind,
            onmounted: move |event| common.mounted(event),
            ..data,
            CardBody { view, on_edit }
            CardActions { actions, focused, on_action }
            CardFooterView { footer }
        }
    }
}
