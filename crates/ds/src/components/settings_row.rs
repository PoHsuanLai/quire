//! SettingsRow: a settings-style list row for networks, devices and outputs, usable outside a
//! Menu (sill FINDINGS Q79): a glyph, a title and a detail line in `MenuEntry::Row`'s type (the
//! shell text menu's, design/13 section 13.3.3), and a trailing mark; 44 px high, a hairline
//! between rows. A control-center list and a menu then read alike.
//!
//! A `div[role=button]`, as `ModuleTile`: a toggle row holds a button of its own. Enter or
//! Space on the row runs `onclick` on both renderers (Blitz synthesises no click from a key).

use crate::components::press::{Press, PressListeners};
use crate::components::settings_row_trailing::{RowTrailing, trailing as trailing_mark};
use crate::components::text_runs::{Text, text};
use crate::components::vocab::Availability;
use crate::icon::Icon;
use crate::icon::render::{Glyph, IconSize};
use dioxus::prelude::*;

/// One settings row. `onclick` hears a press on the row (a toggle's own press is the toggle's).
#[component]
pub fn SettingsRow(
    #[props(default)] glyph: Option<Icon>,
    #[props(into)] title: Text,
    detail: Option<Text>,
    #[props(default)] trailing: RowTrailing,
    #[props(default)] availability: Availability,
    onclick: EventHandler<Press>,
) -> Element {
    let live = availability == Availability::Enabled;
    let listen = PressListeners::new(onclick);
    let mark = trailing_mark(&trailing, &title, availability);
    rsx! {
        div {
            class: "ds-settings-row",
            role: "button",
            tabindex: "0",
            "data-trailing": trailing.slug(),
            "aria-pressed": trailing.pressed(),
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
                span { class: "ds-settings-row-glyph",
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
