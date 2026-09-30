//! RadioGroup: one choice out of a few, `NSButton` radio buttons in a group (design/30 section
//! 2.3). Markup: `div.ds-radio-group[role=radiogroup]` of
//! `button.ds-radio-group-item[role=radio][aria-checked]`, each holding
//! `span.ds-radio-group-indicator`, an optional image and `span.ds-radio-group-label`.
//!
//! One item is in the tab order, the checked one; the arrows move to the previous or next
//! enabled item and check it, Home and End go to the ends, Space checks the focused item.

use crate::components::content::icon_view::IconView;
use crate::components::content::text_runs::text;
use crate::components::controls::choice::Choice;
use crate::components::controls::glyph::glyph_size;
use crate::components::controls::press::{ActivationKeys, activates, disabled, use_pressing};
use crate::focus::soon::focus_soon;
use crate::root::common::Common;
use crate::stack::roving::{Rove, Roving, Wrap};
use dioxus::prelude::*;
use ds_core::vocab::{Availability, Check, Selection};
use ds_core::word::Word;
use ds_style::tokens::control_size::ControlSize;
use std::rc::Rc;

/// How the items are laid out (`data-arrangement`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum Arrangement {
    /// One under the other.
    #[default]
    Column,
    /// Side by side, each item's image over its label: the Appearance choice.
    Row,
}

/// The item after `key` moved from `at`: the enabled neighbour the arrows and Home and End name.
fn rove_to<T: Clone + PartialEq>(choices: &[Choice<T>], at: &T, rove: Rove) -> Option<T> {
    let items = choices
        .iter()
        .map(|choice| (choice.value.clone(), choice.availability))
        .collect();
    Roving::new(items, Wrap::Stops)
        .focus(at)
        .rove(rove)
        .focused()
        .cloned()
        .filter(|to| to != at)
}

/// Whether `value` is the checked one.
fn checked<T: PartialEq>(choice: &Choice<T>, value: &T) -> Check {
    match Selection::of(&choice.value, value) {
        Selection::Selected => Check::On,
        Selection::Unselected => Check::Off,
    }
}

/// A group of radio buttons: `choices`, of which `value` is checked. `label` names the group.
/// A choice's image draws above its label under `Arrangement::Row`, before it otherwise.
#[component]
pub fn RadioGroup<T: Clone + PartialEq + 'static>(
    label: String,
    choices: Vec<Choice<T>>,
    value: T,
    #[props(default)] arrangement: Arrangement,
    #[props(default)] size: ControlSize,
    #[props(default)] availability: Availability,
    onchange: EventHandler<T>,
    #[props(default)] common: Common,
) -> Element {
    let mut items = use_signal(Vec::<Option<Rc<MountedData>>>::new);
    let class = common.class("ds-radio-group");
    let data = common.data_attributes();
    let live = availability == Availability::Enabled;
    let values: Vec<T> = choices.iter().map(|choice| choice.value.clone()).collect();
    let keyed = choices.clone();
    let current = value.clone();
    rsx! {
        div {
            id: common.id.clone(),
            class,
            role: "radiogroup",
            "aria-label": common.aria_label.clone().unwrap_or(label),
            "data-arrangement": arrangement.slug(),
            "data-size": size.slug(),
            "data-availability": availability.slug(),
            "aria-disabled": availability.aria_disabled(),
            "aria-busy": availability.aria_busy(),
            onkeydown: move |event| {
                let Some(rove) = Rove::of(&event.key()) else { return };
                if !live {
                    return;
                }
                if let Some(to) = rove_to(&keyed, &current, rove) {
                    event.prevent_default();
                    event.stop_propagation();
                    if let Some(index) = values.iter().position(|value| *value == to) {
                        if let Some(Some(element)) = items.peek().get(index) {
                            focus_soon(element.clone());
                        }
                    }
                    onchange.call(to);
                }
            },
            onmounted: move |event| common.mounted(event),
            ..data,
            for (index , choice) in choices.into_iter().enumerate() {
                RadioItem {
                    key: "{index}",
                    state: checked(&choice, &value),
                    enabled: choice.availability,
                    group: availability,
                    choice: choice.clone(),
                    size,
                    onpick: {
                        let picked = choice.value.clone();
                        move |()| onchange.call(picked.clone())
                    },
                    onmounted: move |event: MountedEvent| {
                        let mut held = items.write();
                        if held.len() <= index {
                            held.resize(index + 1, None);
                        }
                        held[index] = Some(event.data());
                    },
                }
            }
        }
    }
}

/// One radio button: the indicator, the image and the label of `choice`. `enabled` is the
/// choice's own availability and `group` the group's: it takes input only when both do.
#[component]
fn RadioItem<T: Clone + PartialEq + 'static>(
    choice: Choice<T>,
    state: Check,
    enabled: Availability,
    group: Availability,
    size: ControlSize,
    onpick: EventHandler<()>,
    onmounted: EventHandler<MountedEvent>,
) -> Element {
    let pressing = use_pressing();
    let live = enabled == Availability::Enabled && group == Availability::Enabled;
    rsx! {
        button {
            r#type: "button",
            class: "ds-radio-group-item",
            role: "radio",
            "aria-checked": state.aria(),
            "data-state": state.slug(),
            "data-availability": enabled.slug(),
            "data-pressed": if live { pressing.attr() } else { None },
            "aria-disabled": enabled.aria_disabled(),
            disabled: disabled(if group == Availability::Enabled { enabled } else { group }),
            tabindex: if state == Check::On { "0" } else { "-1" },
            onmounted: move |event| onmounted.call(event),
            onmousedown: move |event| pressing.pointer_down(&event),
            onmouseleave: move |_| pressing.released(),
            onmouseup: move |_| pressing.released(),
            onblur: move |_| pressing.released(),
            onkeyup: move |_| pressing.released(),
            onkeydown: move |event| {
                if live && activates(&event, ActivationKeys::SpaceOnly) {
                    event.prevent_default();
                    event.stop_propagation();
                    pressing.key_down(&event, ActivationKeys::SpaceOnly);
                    onpick.call(());
                }
            },
            onclick: move |_| {
                if live {
                    onpick.call(());
                }
            },
            span { class: "ds-radio-group-indicator", "aria-hidden": "true",
                span { class: "ds-radio-group-dot" }
            }
            if let Some(source) = choice.icon {
                span { class: "ds-radio-group-image",
                    IconView { source, size: glyph_size(size) }
                }
            }
            span { class: "ds-radio-group-label", {text(&choice.label)} }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{checked, rove_to};
    use crate::components::controls::choice::Choice;
    use crate::stack::roving::Rove;
    use crate::stack::roving::{Edge, Step};
    use ds_core::vocab::{Availability, Check};

    fn choices() -> Vec<Choice<u8>> {
        vec![
            Choice::new(0, "a"),
            Choice::new(1, "b").with_availability(Availability::Disabled),
            Choice::new(2, "c"),
        ]
    }

    #[test]
    fn the_arrows_skip_a_disabled_choice_and_stop_at_the_ends() {
        // (from, rove, to)
        const CASES: &[(u8, Rove, Option<u8>)] = &[
            (0, Rove::Step(Step::Down), Some(2)),
            (2, Rove::Step(Step::Up), Some(0)),
            (2, Rove::Step(Step::Down), None),
            (0, Rove::Step(Step::Up), None),
            (2, Rove::Edge(Edge::First), Some(0)),
            (0, Rove::Edge(Edge::Last), Some(2)),
            (0, Rove::Edge(Edge::First), None),
        ];
        for &(from, rove, want) in CASES {
            assert_eq!(rove_to(&choices(), &from, rove), want, "{from} {rove:?}");
        }
    }

    #[test]
    fn only_the_value_is_checked() {
        let choices = choices();
        let states: Vec<Check> = choices.iter().map(|choice| checked(choice, &2)).collect();
        assert_eq!(states, [Check::Off, Check::Off, Check::On]);
    }
}
