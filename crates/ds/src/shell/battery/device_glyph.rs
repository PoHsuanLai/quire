//! DeviceGlyph: a device's filled glyph, for the battery widget's rings (design/23-WIDGETS.md
//! section 4.4; the user's pick 2026-09-27, "filled device glyphs"). The reference's Batteries
//! widget draws each device as a filled symbol (M12); ours are abstract solids on the Lucide 24
//! grid (`device_forms.rs`), one path in `currentColor`, no stroke, so they sit in the same
//! `svg.ds-ic` box as every other glyph and size the same way.

use crate::shell::battery::device_forms::{form_of, path_of};
use crate::style::icon::render::IconSize;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

/// A device that reports a battery.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Device {
    /// A laptop: this computer, when it has a battery.
    #[default]
    Laptop,
    /// A desktop computer.
    Desktop,
    /// A phone.
    Phone,
    /// A tablet.
    Tablet,
    /// A watch.
    Watch,
    /// Over-ear headphones.
    Headphones,
    /// Earbuds.
    Earbuds,
    /// A mouse or a trackpad.
    Mouse,
    /// A keyboard.
    Keyboard,
    /// A speaker.
    Speaker,
    /// A game controller.
    Gamepad,
    /// Anything else with a battery.
    Other,
}

impl Device {
    /// Every device, in the order the gallery shows them.
    pub const ALL: [Device; 12] = [
        Device::Laptop,
        Device::Desktop,
        Device::Phone,
        Device::Tablet,
        Device::Watch,
        Device::Headphones,
        Device::Earbuds,
        Device::Mouse,
        Device::Keyboard,
        Device::Speaker,
        Device::Gamepad,
        Device::Other,
    ];

    /// The `data-device` word.
    pub fn slug(self) -> &'static str {
        match self {
            Device::Laptop => "laptop",
            Device::Desktop => "desktop",
            Device::Phone => "phone",
            Device::Tablet => "tablet",
            Device::Watch => "watch",
            Device::Headphones => "headphones",
            Device::Earbuds => "earbuds",
            Device::Mouse => "mouse",
            Device::Keyboard => "keyboard",
            Device::Speaker => "speaker",
            Device::Gamepad => "gamepad",
            Device::Other => "other",
        }
    }
}

/// `svg.ds-ic.ds-device-glyph[data-device]`: `device` filled in `currentColor` at `size`.
/// Decorative (`aria-hidden`): the ring it sits in carries the device's name.
#[component]
pub fn DeviceGlyph(device: Device, #[props(default)] size: IconSize) -> Element {
    let px = size.px();
    let form = form_of(device);
    rsx! {
        svg {
            class: "ds-ic ds-device-glyph",
            "data-device": device.slug(),
            "data-size": "{px}",
            width: "{px}",
            height: "{px}",
            view_box: "0 0 24 24",
            "aria-hidden": "true",
            "fill": "currentColor",
            "stroke": "none",
            path { d: path_of(form), "fill-rule": form.rule.slug() }
        }
    }
}
