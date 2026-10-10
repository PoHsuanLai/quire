//! ShortcutField: the control that records a keyboard shortcut (System Settings' shortcut
//! recorder). It shows the chord it was given; a click, or Return or Space with the keyboard on
//! it, makes it listen ("Type shortcut"); the next combination is offered to the caller, Escape
//! gives up, Backspace or Delete clears. While it listens it keeps every key it hears from the
//! window's actions.
//!
//! Markup: `span.ds-shortcut-field[data-recording][data-clash]` holding
//! `button.ds-shortcut-field-well` and, when another action uses the chord, a
//! `span.ds-shortcut-field-clash`.

use crate::components::controls::press::{ActivationKeys, activates, disabled};
use crate::components::fields::shortcut_field_model::{
    Capture, Heard, ShortcutClash, ShortcutRecorded, hear,
};
use crate::keys::use_platform;
use crate::root::common::Common;
use chordkit::Chord;
use dioxus::prelude::*;
use ds_core::command::chord_text;
use ds_core::vocab::Availability;
use ds_core::word::Word;

/// What the well reads while it is not listening.
const EMPTY: &str = "Add shortcut";
/// What the well reads while it listens.
const LISTENING: &str = "Type shortcut";

/// A shortcut recorder. `value` is the shortcut shown (the caller holds it and stores what
/// `onrecord` reports); `label` names it to assistive technology. `clash` names the action that
/// already uses the shortcut, shown under the field.
#[component]
pub fn ShortcutField(
    label: String,
    #[props(default)] value: Option<Chord>,
    onrecord: EventHandler<ShortcutRecorded>,
    #[props(default)] clash: ShortcutClash,
    #[props(default)] availability: Availability,
    #[props(default)] common: Common,
) -> Element {
    let platform = use_platform();
    let mut capture = use_signal(Capture::default);
    let live = availability == Availability::Enabled;
    let mut finish = move |recorded: ShortcutRecorded| {
        capture.set(Capture::Idle);
        onrecord.call(recorded);
    };
    let text = match (capture(), value) {
        (Capture::Listening, _) => LISTENING.to_owned(),
        (Capture::Idle, Some(chord)) => chord_text(platform, &chord),
        (Capture::Idle, None) => EMPTY.to_owned(),
    };
    let class = common.class("ds-shortcut-field");
    let data = common.data_attributes();
    let name = common.aria_label.clone().unwrap_or(label);
    let used_by = match &clash {
        ShortcutClash::None => None,
        ShortcutClash::With(action) => Some(format!("Used by {action}")),
    };
    rsx! {
        span {
            class,
            "data-recording": capture().slug(),
            "data-clash": used_by.is_some().then_some("with"),
            "data-availability": availability.slug(),
            ..data,
            button {
                r#type: "button",
                class: "ds-shortcut-field-well",
                id: common.id.clone(),
                "aria-label": "{name}",
                "aria-pressed": (capture() == Capture::Listening).then_some("true"),
                disabled: disabled(availability),
                onclick: move |_| {
                    if live {
                        capture.set(Capture::Listening);
                    }
                },
                onblur: move |_| {
                    if capture() == Capture::Listening {
                        finish(ShortcutRecorded::Cancelled);
                    }
                },
                onkeydown: move |event: KeyboardEvent| match capture() {
                    Capture::Idle => {
                        if live && activates(&event, ActivationKeys::ReturnAndSpace) {
                            event.prevent_default();
                            event.stop_propagation();
                            capture.set(Capture::Listening);
                        }
                    }
                    Capture::Listening => {
                        let heard = hear(platform, &event.key(), event.modifiers());
                        if heard == Heard::Leave {
                            return;
                        }
                        event.prevent_default();
                        event.stop_propagation();
                        match heard {
                            Heard::Offer(chord) => finish(ShortcutRecorded::Chord(chord)),
                            Heard::Clear => finish(ShortcutRecorded::Cleared),
                            Heard::Cancel => finish(ShortcutRecorded::Cancelled),
                            Heard::Wait | Heard::Leave => {}
                        }
                    }
                },
                onmounted: move |event| common.mounted(event),
                "{text}"
            }
            if let Some(used_by) = &used_by {
                span { class: "ds-shortcut-field-clash", role: "status", "{used_by}" }
            }
        }
    }
}
