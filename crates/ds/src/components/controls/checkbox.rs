//! Checkbox: `NSButton` as a checkbox (design/30 section 2.3). A box that is off, on or mixed,
//! and its label; a press on either flips it, and so does Space.
//! Markup: `button.ds-checkbox[role=checkbox][aria-checked][data-state][data-size]` holding
//! `span.ds-checkbox-indicator` (the box, with the check mark in it) and
//! `span.ds-checkbox-label`.

use crate::components::content::text_runs::{TextLine, text};
use crate::components::controls::press::{ActivationKeys, activates, disabled, use_pressing};
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::vocab::{Availability, Check};
use ds_core::word::Word;
use ds_style::icon::Icon;
use ds_style::icon::render::{Glyph, IconSize};
use ds_style::tokens::control_size::ControlSize;

/// The check mark's size inside a box of `size`: the smallest glyphs the box holds.
fn mark_size(size: ControlSize) -> IconSize {
    match size {
        ControlSize::Mini | ControlSize::Small => IconSize::Micro,
        ControlSize::Regular | ControlSize::Large => IconSize::Tiny,
    }
}

/// A checkbox with `label`. A press on a mixed one turns it on, as `NSButton` does
/// (`Check::flipped`). `Busy` takes no input, as `Disabled` does, and writes `aria-busy`.
#[component]
pub fn Checkbox(
    #[props(into)] label: TextLine,
    value: Check,
    #[props(default)] size: ControlSize,
    #[props(default)] availability: Availability,
    onchange: EventHandler<Check>,
    #[props(default)] common: Common,
) -> Element {
    let pressing = use_pressing();
    let live = availability == Availability::Enabled;
    let class = common.class("ds-checkbox");
    let data = common.data_attributes();
    rsx! {
        button {
            r#type: "button",
            id: common.id.clone(),
            class,
            role: "checkbox",
            "aria-checked": value.aria(),
            "data-state": value.slug(),
            "data-size": size.slug(),
            "data-availability": availability.slug(),
            "data-pressed": if live { pressing.attr() } else { None },
            "aria-label": common.aria_label.clone(),
            "aria-disabled": availability.aria_disabled(),
            "aria-busy": availability.aria_busy(),
            disabled: disabled(availability),
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
                    onchange.call(value.flipped());
                }
            },
            onclick: move |_| {
                if live {
                    onchange.call(value.flipped());
                }
            },
            onmounted: move |event| common.mounted(event),
            ..data,
            span { class: "ds-checkbox-indicator", "aria-hidden": "true",
                match value {
                    Check::On => rsx! {
                        Glyph { icon: Icon::Check, size: mark_size(size) }
                    },
                    Check::Mixed => rsx! {
                        span { class: "ds-checkbox-dash" }
                    },
                    Check::Off => rsx! {},
                }
            }
            span { class: "ds-checkbox-label", {text(&label)} }
        }
    }
}
