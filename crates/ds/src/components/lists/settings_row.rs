//! SettingsRow: a settings-style list row for networks, devices and outputs, usable outside a
//! Menu: a glyph, a title and a detail line in `MenuEntry::Row`'s type (the
//! shell text menu's, design/13 section 13.3.3), and a trailing mark; 44 px high, a hairline
//! between rows. A control-center list and a menu then read alike.
//!
//! A `div[role=button]`, as `ModuleTile`: a toggle row holds a button of its own. Enter or
//! Space on the row runs `onclick` on both renderers (Blitz synthesises no click from a key).
//!
//! An operation on the row's item (design/26-DETAILS.md 5.2.2, 5.2.3, 5.2.8) is its `phase`:
//! while it is pending a spinner takes the trailing slot ([`ring`]); its end draws the row as it
//! is.

use crate::components::content::text_runs::{TextLine, text};
use crate::components::controls::press::PressListeners;
use crate::components::controls::spinner::{SPIN, ring};
use crate::components::lists::settings_row_phase::{RowDisc, RowPhase};
use crate::components::lists::settings_row_trailing::{RowTrailing, trailing as trailing_mark};
use crate::motion::detail::{
    pending::PendingFrame, touch::Touch, use_detail::use_detail, use_operation::use_operation,
    use_pending::use_pending,
};
use dioxus::prelude::*;
use ds_core::press::Press;
use ds_core::vocab::Availability;
use ds_style::icon::Icon;
use ds_style::icon::render::{Glyph, IconSize};

/// One settings row. `onclick` hears a press on the row (a toggle's own press is the toggle's).
///
/// `phase` is the operation on the row's item ([`RowPhase`], default `Rest`), `disc` the disc the
/// glyph sits on (default none).
#[component]
pub fn SettingsRow(
    #[props(default)] glyph: Option<Icon>,
    #[props(into)] title: TextLine,
    detail: Option<TextLine>,
    #[props(default)] trailing: RowTrailing,
    #[props(default)] availability: Availability,
    onclick: EventHandler<Press>,
    #[props(default)] phase: RowPhase,
    #[props(default)] disc: RowDisc,
) -> Element {
    let live = availability == Availability::Enabled;
    let listen = PressListeners::new(onclick);
    let moment = use_detail(phase, Touch::Remote);
    let frame = use_pending(use_operation(moment.cue()), SPIN);
    let mark = match frame {
        PendingFrame::Step(_) => pending_trail(frame),
        PendingFrame::Idle => trailing_mark(&trailing, &title, availability),
    };
    rsx! {
        div {
            class: "ds-settings-row",
            role: "button",
            tabindex: "0",
            "data-trailing": trailing.slug(),
            "data-phase": phase_slug(phase),
            "aria-pressed": trailing.pressed(),
            "aria-busy": busy(phase),
            "aria-disabled": availability.aria_disabled(),
            onclick: move |event| {
                if live {
                    listen.click(&event);
                }
            },
            onkeydown: move |event| {
                let key = event.key();
                if live && (key == Key::Enter || key == Key::Character(" ".into())) {
                    event.prevent_default();
                    onclick.call(Press::primary());
                }
            },
            if let Some(icon) = glyph {
                span { class: "ds-settings-row-glyph", "data-disc": disc.attr(),
                    Glyph { icon, size: IconSize::Base }
                }
            }
            span { class: "ds-settings-row-words",
                span { class: "ds-settings-row-title ds-truncate", {text(&title)} }
                if let Some(detail) = detail {
                    span { class: "ds-settings-row-detail ds-truncate", {text(&detail)} }
                }
            }
            {mark}
        }
    }
}

/// The trailing slot while a spinner holds it: a 14 px well the ring turns around.
fn pending_trail(frame: PendingFrame) -> Element {
    rsx! {
        span { class: "ds-settings-row-trail", "data-mark": "pending",
            span { class: "ds-settings-row-spin", {ring(frame)} }
        }
    }
}

/// The `data-phase` word: written only while something is happening.
fn phase_slug(phase: RowPhase) -> Option<&'static str> {
    match phase {
        RowPhase::Rest => None,
        RowPhase::Pending(_) => Some("pending"),
        RowPhase::Succeeded(_) => Some("succeeded"),
        RowPhase::Failed(_) => Some("failed"),
    }
}

/// `aria-busy`: while the operation runs.
fn busy(phase: RowPhase) -> Option<&'static str> {
    match phase {
        RowPhase::Pending(_) => Some("true"),
        RowPhase::Rest | RowPhase::Succeeded(_) | RowPhase::Failed(_) => None,
    }
}
