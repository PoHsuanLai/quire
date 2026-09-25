//! What a `SettingsRow` ends in (sill FINDINGS Q79): nothing, a check, a toggle, a chevron, a
//! value, or a glyph. The toggle is fenced: its press and its keys stay inside it, so flipping
//! a device's switch never also runs the row.

use crate::components::text_runs::{Text, text};
use crate::components::toggle::Toggle;
use crate::components::vocab::{Availability, Switch};
use crate::icon::Icon;
use crate::icon::render::{Glyph, IconSize};
use dioxus::prelude::*;

/// A settings row's trailing mark.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum RowTrailing {
    /// Nothing after the words.
    #[default]
    None,
    /// A check when `On`: the network in use, the output chosen. `Off` keeps the column empty.
    Check(Switch),
    /// A switch of its own: `value` shown, `on_toggle` hearing the flipped value.
    Toggle {
        /// Whether it is on.
        value: Switch,
        /// The switch was flipped; the row's own `onclick` does not run.
        on_toggle: EventHandler<Switch>,
    },
    /// A chevron: the row opens something further.
    Chevron,
    /// A value in the faint detail type: "Connected", "84%".
    Text(Text),
    /// A glyph in the faint ink: a lock on a secured network.
    Glyph(Icon),
}

impl RowTrailing {
    /// The `data-trailing` word.
    pub(crate) fn slug(&self) -> Option<&'static str> {
        match self {
            RowTrailing::None => None,
            RowTrailing::Check(_) => Some("check"),
            RowTrailing::Toggle { .. } => Some("toggle"),
            RowTrailing::Chevron => Some("chevron"),
            RowTrailing::Text(_) => Some("text"),
            RowTrailing::Glyph(_) => Some("glyph"),
        }
    }

    /// `aria-pressed` for a check row: whether it is the chosen one.
    pub(crate) fn pressed(&self) -> Option<&'static str> {
        match self {
            RowTrailing::Check(Switch::On) => Some("true"),
            RowTrailing::Check(Switch::Off) => Some("false"),
            _ => None,
        }
    }
}

/// The trailing mark drawn, `title` naming a toggle.
pub(crate) fn trailing(mark: &RowTrailing, title: &Text, availability: Availability) -> Element {
    match mark.clone() {
        RowTrailing::None => rsx! {},
        RowTrailing::Check(Switch::On) => rsx! {
            span { class: "ds-settings-row-trail", "data-mark": "check",
                Glyph { icon: Icon::Check, size: IconSize::Compact }
            }
        },
        RowTrailing::Check(Switch::Off) => rsx! {
            span { class: "ds-settings-row-trail", "data-mark": "check" }
        },
        RowTrailing::Toggle { value, on_toggle } => toggle(value, on_toggle, title, availability),
        RowTrailing::Chevron => rsx! {
            span { class: "ds-settings-row-trail",
                Glyph { icon: Icon::ChevronRight, size: IconSize::Compact }
            }
        },
        RowTrailing::Text(value) => rsx! {
            span { class: "ds-settings-row-trail ds-settings-row-value", {text(&value)} }
        },
        RowTrailing::Glyph(icon) => rsx! {
            span { class: "ds-settings-row-trail",
                Glyph { icon, size: IconSize::Compact }
            }
        },
    }
}

/// The row's switch, fenced so its click and keys are its own.
fn toggle(
    value: Switch,
    on_toggle: EventHandler<Switch>,
    title: &Text,
    availability: Availability,
) -> Element {
    let label = title.plain_text();
    rsx! {
        span {
            class: "ds-settings-row-trail",
            "data-mark": "toggle",
            onclick: move |event| event.stop_propagation(),
            onkeydown: move |event| event.stop_propagation(),
            Toggle { label, value, availability, onchange: on_toggle }
        }
    }
}
