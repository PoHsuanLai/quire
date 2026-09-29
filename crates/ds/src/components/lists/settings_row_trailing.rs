//! What a `SettingsRow` ends in: nothing, a check, a toggle, a chevron, a
//! value, a glyph, or a device's battery. The toggle is fenced: its press and its keys stay inside
//! it, so flipping a device's switch never also runs the row. A check that arrives with a success
//! draws on (`Settle{Check}`, design/26).

use crate::components::content::text_runs::{TextLine, text};
use crate::components::controls::toggle::Toggle;
use crate::components::lists::row_battery::RowBattery;
use crate::core::vocab::{Availability, Fraction, Switch};
use crate::focus::click::kept_click;
use crate::motion::detail::{check_mark::CheckMark, first_show::FirstShow, settle::Settling};
use crate::style::icon::Icon;
use crate::style::icon::render::{Glyph, IconSize};
use crate::style::tokens::control_size::ControlSize;
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
    Text(TextLine),
    /// A glyph in the faint ink: a lock on a secured network.
    Glyph(Icon),
    /// A connected device's battery: its glyph and percentage, sweeping in from empty with the
    /// count in step when it first shows (design/26), then moving from where it is.
    Battery(Fraction),
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
            RowTrailing::Battery(_) => Some("battery"),
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

/// The trailing mark drawn, `title` naming a toggle; a check draws on while `settling` says; a
/// battery sweeps in as `first` says.
pub(crate) fn trailing(
    mark: &RowTrailing,
    title: &TextLine,
    availability: Availability,
    settling: Settling,
    first: FirstShow,
) -> Element {
    match mark.clone() {
        RowTrailing::None => rsx! {},
        RowTrailing::Check(Switch::On) => rsx! {
            span { class: "ds-settings-row-trail", "data-mark": "check",
                if let Settling::Drawing(_) = settling {
                    CheckMark { settling, size: IconSize::Compact }
                } else {
                    Glyph { icon: Icon::Check, size: IconSize::Compact }
                }
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
        RowTrailing::Battery(level) => rsx! {
            span { class: "ds-settings-row-trail ds-settings-row-value", "data-mark": "battery",
                RowBattery { level, first }
            }
        },
    }
}

/// The row's switch, fenced so its click and keys are its own: the Small switch (26 x 15), the
/// reference's mini switch in form rows (design/29-SIZING.md section 13 decision 4).
fn toggle(
    value: Switch,
    on_toggle: EventHandler<Switch>,
    title: &TextLine,
    availability: Availability,
) -> Element {
    let label = title.plain_text();
    rsx! {
        span {
            class: "ds-settings-row-trail",
            "data-mark": "toggle",
            onclick: move |event| {
                event.stop_propagation();
                kept_click(&event);
            },
            onkeydown: move |event| event.stop_propagation(),
            Toggle { label, value, size: ControlSize::Mini, availability, onchange: on_toggle }
        }
    }
}
