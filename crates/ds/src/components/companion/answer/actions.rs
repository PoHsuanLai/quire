//! A card's buttons, drawn: the action's words and rank, its key, and for an outbound, running
//! or destructive effect the tag that says what it does, so no such action reads as plain.

use super::action::{ActionRole, CardAction, CardActionId};
use crate::components::companion::effect::view::EffectTag;
use crate::components::controls::button::Button;
use crate::components::controls::button_model::{Answers, ButtonRole};
use dioxus::prelude::*;
use ds_core::vocab::EffectMark;
use ds_core::word::Word;

/// The buttons of a card in a row. `focused` is the index of the one with the keyboard; the
/// card's primary action answers Return.
#[component]
pub fn CardActions(
    actions: Vec<CardAction>,
    #[props(default)] focused: Option<usize>,
    on_action: EventHandler<CardActionId>,
) -> Element {
    if actions.is_empty() {
        return rsx! {};
    }
    rsx! {
        div { class: "ds-card-actions", role: "group", "aria-label": "Actions",
            for (index , action) in actions.into_iter().enumerate() {
                CardActionButton {
                    key: "{index}",
                    action,
                    focus: focus_of(focused, index),
                    on_action,
                }
            }
        }
    }
}

/// Whether the keyboard starts on the action at `index`.
fn focus_of(focused: Option<usize>, index: usize) -> bool {
    focused == Some(index)
}

/// How an action's rank draws: Primary is the default button, Destructive is never one.
fn rank(role: ActionRole) -> (ButtonRole, Answers) {
    match role {
        ActionRole::Primary => (ButtonRole::Normal, Answers::Return),
        ActionRole::Secondary => (ButtonRole::Normal, Answers::Nothing),
        ActionRole::Destructive => (ButtonRole::Destructive, Answers::Nothing),
    }
}

/// Whether the effect gets its tag: reading and an undoable change are quiet.
fn tagged(effect: EffectMark) -> bool {
    !matches!(effect, EffectMark::Read | EffectMark::UndoableWrite)
}

#[component]
fn CardActionButton(
    action: CardAction,
    focus: bool,
    on_action: EventHandler<CardActionId>,
) -> Element {
    let (role, answers) = rank(action.role);
    let id = action.id.clone();
    let title = action.keys.as_ref().map(|_| action.label.clone());
    rsx! {
        span {
            class: "ds-card-action",
            "data-role": action.role.slug(),
            "data-effect": action.effect.slug(),
            "data-focus": focus.then_some("keyboard"),
            Button {
                label: action.label.clone(),
                role,
                answers,
                autofocus: focus,
                availability: action.availability,
                title,
                title_shortcut: action.keys.clone(),
                onclick: move |_| on_action.call(id.clone()),
            }
            if tagged(action.effect) {
                EffectTag { effect: action.effect }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{rank, tagged};
    use crate::components::controls::button_model::Answers;
    use ds_core::vocab::EffectMark;

    #[test]
    fn only_the_primary_answers_return_and_only_loud_effects_are_tagged() {
        use super::ActionRole::{Destructive, Primary, Secondary};
        assert_eq!(rank(Primary).1, Answers::Return);
        assert_eq!(rank(Secondary).1, Answers::Nothing);
        assert_eq!(rank(Destructive).1, Answers::Nothing);
        assert!(tagged(EffectMark::Outbound) && tagged(EffectMark::Destructive));
        assert!(!tagged(EffectMark::Read) && !tagged(EffectMark::UndoableWrite));
    }
}
