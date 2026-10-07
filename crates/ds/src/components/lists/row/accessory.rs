//! What ends a row: its trailing accessory (design/30 section 2.6). A busy row shows the small
//! spinner in its place; a toggle's press and keys stay inside it, so flipping a row's switch
//! never also runs the row.

use crate::components::content::status::battery::BatteryGlyph;
use crate::components::content::status::battery_state::BatteryState;
use crate::components::controls::badge::label as count_label;
use crate::components::controls::progress::model::{Progress, ProgressStyle};
use crate::components::controls::progress::view::ProgressIndicator;
use crate::components::controls::toggle::Toggle;
use crate::focus::click::kept_click;
use dioxus::prelude::*;
use ds_core::vocab::{Availability, Check};
use ds_motion::detail::operation::{Operation, PendingToken};
use ds_style::icon::Icon;
use ds_style::icon::render::{Glyph, IconSize};
use ds_style::tokens::control_size::ControlSize;

/// A row's trailing accessory.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Accessory {
    /// Nothing after the words.
    #[default]
    None,
    /// A check while `On` (the network in use, the output chosen), a dash while `Mixed`; `Off`
    /// keeps the column empty.
    Check(Check),
    /// A switch of its own: `value` shown, `on_toggle` hearing the flipped value. The row's own
    /// `onclick` does not run.
    Toggle {
        /// Whether it is on.
        value: Check,
        /// The switch was flipped.
        on_toggle: EventHandler<Check>,
    },
    /// A chevron: the row opens something further.
    Chevron,
    /// A value in the faint detail type: "Connected", "84%", a shortcut's glyphs.
    Text(String),
    /// A glyph in the faint ink: a lock on a secured network.
    Glyph(Icon),
    /// A device's battery: its glyph and percentage.
    Battery(BatteryState),
    /// The small spinner, for work the row does not otherwise show (a busy row shows it itself).
    Spinner,
    /// A count as plain text, no capsule (macOS Mail's sidebar): semibold, tabular, in the
    /// secondary ink; more than 999 reads `999+`, zero draws nothing.
    Badge(u32),
    /// A control the caller draws (a `⋯` button, a cancel button). Presses and keys in it stay
    /// in it: the row neither runs nor takes the selection.
    Slot(Element),
}

impl Accessory {
    /// The `data-trailing` word.
    pub(crate) fn slug(&self) -> Option<&'static str> {
        match self {
            Accessory::None => None,
            Accessory::Check(_) => Some("check"),
            Accessory::Toggle { .. } => Some("toggle"),
            Accessory::Chevron => Some("chevron"),
            Accessory::Text(_) => Some("text"),
            Accessory::Glyph(_) => Some("glyph"),
            Accessory::Battery(_) => Some("battery"),
            Accessory::Spinner => Some("spinner"),
            Accessory::Badge(_) => Some("badge"),
            Accessory::Slot(_) => Some("slot"),
        }
    }

    /// The row's `aria-checked`: whether a check row is the chosen one.
    pub(crate) fn checked(&self) -> Option<&'static str> {
        match self {
            Accessory::Check(check) => Some(check.aria()),
            _ => None,
        }
    }
}

/// The small spinner: it turns from the moment it is drawn, which is the moment the work it
/// stands for is shown, and it fades in over `--t-quick` (`row.css`, design/30 section 2.9).
#[component]
fn Busy() -> Element {
    let operation = use_hook(|| Operation::Running(PendingToken::start()));
    rsx! {
        span { class: "ds-row-spin", ProgressIndicator { style: ProgressStyle::Spinner, progress: Progress::Unknown(operation), size: ControlSize::Mini } }
    }
}

/// The accessory drawn. `label` names a toggle; `availability` decides what a busy row shows.
pub(crate) fn draw(accessory: &Accessory, label: &str, availability: Availability) -> Element {
    if availability == Availability::Busy {
        return rsx! {
            span { class: "ds-row-trailing", "data-mark": "busy", Busy {} }
        };
    }
    match accessory.clone() {
        Accessory::None => rsx! {},
        Accessory::Check(Check::On) => rsx! {
            span { class: "ds-row-trailing", "data-mark": "check",
                Glyph { icon: Icon::Check, size: IconSize::Compact }
            }
        },
        Accessory::Check(Check::Mixed) => rsx! {
            span { class: "ds-row-trailing", "data-mark": "check",
                Glyph { icon: Icon::Minus, size: IconSize::Compact }
            }
        },
        Accessory::Check(Check::Off) => rsx! {
            span { class: "ds-row-trailing", "data-mark": "check" }
        },
        Accessory::Toggle { value, on_toggle } => {
            toggle(value, on_toggle, label.to_owned(), availability)
        }
        Accessory::Chevron => rsx! {
            span { class: "ds-row-trailing", "data-mark": "chevron",
                Glyph { icon: Icon::ChevronRight, size: IconSize::Tiny }
            }
        },
        Accessory::Text(value) => rsx! {
            span { class: "ds-row-trailing", "data-mark": "text", "{value}" }
        },
        Accessory::Glyph(icon) => rsx! {
            span { class: "ds-row-trailing", "data-mark": "glyph",
                Glyph { icon, size: IconSize::Compact }
            }
        },
        Accessory::Battery(state) => rsx! {
            span { class: "ds-row-trailing", "data-mark": "battery",
                BatteryGlyph { state, size: IconSize::Compact }
                span { class: "ds-row-figure", "{state.level.whole_percent()}%" }
            }
        },
        Accessory::Spinner => rsx! {
            span { class: "ds-row-trailing", "data-mark": "busy", Busy {} }
        },
        Accessory::Badge(0) => rsx! {},
        Accessory::Badge(value) => rsx! {
            span { class: "ds-row-trailing", "data-mark": "badge",
                span { class: "ds-row-count", "{count_label(value)}" }
            }
        },
        Accessory::Slot(element) => rsx! {
            span {
                class: "ds-row-trailing",
                "data-mark": "slot",
                "data-slot": "trailing",
                onclick: move |event| {
                    event.stop_propagation();
                    kept_click(&event);
                },
                onmousedown: move |event| event.stop_propagation(),
                {element}
            }
        },
    }
}

/// The row's switch, fenced so its click and keys are its own: the Mini switch (26 x 15), the
/// reference's switch in form rows (design/29-SIZING.md section 13 decision 4).
fn toggle(
    value: Check,
    on_toggle: EventHandler<Check>,
    label: String,
    availability: Availability,
) -> Element {
    rsx! {
        span {
            class: "ds-row-trailing",
            "data-mark": "toggle",
            onclick: move |event| {
                event.stop_propagation();
                kept_click(&event);
            },
            // The switch takes Space itself; Return and Escape are the window's (a sheet's default
            // and cancel buttons), and every other key stays here.
            onkeydown: move |event| {
                if !matches!(event.key(), Key::Enter | Key::Escape) {
                    event.stop_propagation();
                }
            },
            Toggle { label, value, size: ControlSize::Mini, availability, onchange: on_toggle }
        }
    }
}
