//! Stepper: the pair, and with `Readout::Field` the text field it changes (design/30 section
//! 2.1, `NSStepper`). A press on a half steps once at once; held past `DelayToken::RepeatStart`
//! it repeats every `DelayToken::RepeatEvery` until it is let go. The pair takes the keyboard as
//! a spin button: Up and Down step, Home and End go to the ends. The field beside it commits a
//! typed number on Return or when the caret leaves, and the arrow keys in it step too.
//!
//! Markup: `span.ds-stepper[data-size][data-readout]` holding the `TextField` (with `Field`) and
//! `span.ds-stepper-pair[role=spinbutton]` of `span.ds-stepper-up` and `span.ds-stepper-down`;
//! a half that cannot go further is `aria-disabled`, the half held down `data-pressed`.

use crate::components::fields::stepper::hold::{HoldIn, RepeatPace, StepHold};
use crate::components::fields::stepper::model::{Readout, StepDirection, StepRange};
use crate::components::fields::text_field::TextField;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::vocab::{Availability, PressPhase};
use ds_core::word::Word;
use ds_motion::machine::use_machine;
use ds_style::icon::Icon;
use ds_style::icon::render::{Glyph, IconPx, IconSize};
use ds_style::tokens::control_size::ControlSize;

/// The arrow a half draws: small enough that two of them fit in the rung's height.
fn arrow_size(size: ControlSize) -> IconSize {
    let half = size.scale().height.0 / 2;
    IconSize::Px(IconPx(
        u8::try_from(half.saturating_sub(2)).unwrap_or(u8::MAX),
    ))
}

/// The direction a key asks for, if it is one of the arrows.
fn arrow(key: &Key) -> Option<StepDirection> {
    match key {
        Key::ArrowUp => Some(StepDirection::Up),
        Key::ArrowDown => Some(StepDirection::Down),
        _ => None,
    }
}

/// What a step does: the change one step from the value shown, and where it is reported.
#[derive(Clone, Copy)]
struct Stepping {
    value: i32,
    range: StepRange,
    onchange: EventHandler<i32>,
    /// The half-typed text in the field: a step replaces it with the number it lands on.
    draft: Signal<Option<String>>,
}

impl Stepping {
    /// The change one step from the value now.
    fn step(self, direction: StepDirection) {
        self.change(self.range.step_from(self.value, direction));
    }

    /// Report a change that did not come from typing (a step, Home, End): the number shown is
    /// the value now, not whatever was half typed before it.
    fn change(self, to: i32) {
        let mut draft = self.draft;
        draft.set(None);
        self.onchange.call(to);
    }
}

/// A stepper over `range`. `value` is the number it shows and steps; `onchange` hears each
/// step and each typed number, already inside the range. `Busy` and `Disabled` take no input.
#[component]
pub fn Stepper(
    #[props(into)] label: String,
    value: i32,
    range: StepRange,
    #[props(default)] readout: Readout,
    #[props(default)] size: ControlSize,
    #[props(default)] availability: Availability,
    onchange: EventHandler<i32>,
    #[props(default)] common: Common,
) -> Element {
    let mut draft = use_signal(|| None::<String>);
    let stepping = Stepping {
        value,
        range,
        onchange,
        draft,
    };
    let hold = use_machine(
        |_| StepHold::Idle,
        RepeatPace::default(),
        || (),
        move |direction, _| stepping.step(direction),
    );
    let live = availability == Availability::Enabled;
    let pressed = hold.state().read().direction();
    let arrow_size = arrow_size(size);
    let class = common.class("ds-stepper");
    let data = common.data_attributes();
    let on_key = move |event: KeyboardEvent| {
        if !live {
            return;
        }
        if let Some(direction) = arrow(&event.key()) {
            event.prevent_default();
            stepping.step(direction);
        }
    };
    let field_key = on_key;
    let field = match readout {
        Readout::Bare => rsx! {},
        Readout::Field => rsx! {
            TextField {
                label: label.clone(),
                value: draft().unwrap_or_else(|| value.to_string()),
                size,
                availability,
                oninput: move |typed| draft.set(Some(typed)),
                onchange: move |typed: String| {
                    draft.set(None);
                    if let Some(parsed) = range.parse(&typed) {
                        onchange.call(parsed);
                    }
                },
                onkey: field_key,
            }
        },
    };
    let half = move |direction: StepDirection, icon: Icon| {
        let held = (pressed == Some(direction))
            .then_some(PressPhase::Pressed.attr())
            .flatten();
        let spent = (!range.can_step(value, direction)).then_some("true");
        let class = match direction {
            StepDirection::Up => "ds-stepper-up",
            StepDirection::Down => "ds-stepper-down",
        };
        rsx! {
            span {
                class,
                "data-pressed": held,
                "aria-disabled": spent,
                "aria-hidden": "true",
                onpointerdown: move |event: PointerEvent| {
                    if live {
                        event.stop_propagation();
                        hold.send(HoldIn::Press(direction));
                    }
                },
                Glyph { icon, size: arrow_size }
            }
        }
    };
    rsx! {
        span {
            id: common.id.clone(),
            class,
            "data-size": size.slug(),
            "data-readout": readout.slug(),
            "data-availability": availability.slug(),
            "aria-busy": availability.aria_busy(),
            onmounted: move |event| common.mounted(event),
            ..data,
            {field}
            span {
                class: "ds-stepper-pair",
                role: "spinbutton",
                tabindex: if live { Some("0") } else { None },
                "aria-label": label,
                "aria-valuenow": "{value}",
                "aria-valuemin": "{range.min()}",
                "aria-valuemax": "{range.max()}",
                "aria-disabled": availability.aria_disabled(),
                onkeydown: move |event: KeyboardEvent| {
                    match event.key() {
                        Key::Home if live => stepping.change(range.min()),
                        Key::End if live => stepping.change(range.max()),
                        _ => on_key(event),
                    }
                },
                onpointerup: move |_| hold.send(HoldIn::Release),
                onpointerleave: move |_| hold.send(HoldIn::Release),
                onblur: move |_| hold.send(HoldIn::Release),
                {half(StepDirection::Up, Icon::ChevronUp)}
                {half(StepDirection::Down, Icon::ChevronDown)}
            }
        }
    }
}
