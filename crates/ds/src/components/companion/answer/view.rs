//! The answer card component.

use super::action::CardActionId;
use super::model::AnswerView;
use crate::components::companion::form::model::{FieldKeyId, FormValue};
use crate::root::common::Common;
use dioxus::prelude::*;

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
    let _ = (focused, on_action, on_edit);
    let class = common.class("ds-answer-card");
    let data = common.data_attributes();
    let kind = match &view {
        AnswerView::Text(_) => "text",
        AnswerView::DraftReply(_) => "draft-reply",
        AnswerView::ProposedEvent(_) => "proposed-event",
        AnswerView::Plan(_) => "plan",
        AnswerView::Replace(_) => "replace",
        AnswerView::Form(_) => "form",
        AnswerView::Refused(_) => "refused",
    };
    rsx! {
        div {
            id: common.id.clone(),
            class,
            "data-kind": kind,
            onmounted: move |event| common.mounted(event),
            ..data,
        }
    }
}
