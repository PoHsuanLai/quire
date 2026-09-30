//! DeviceBattery: one device's battery in the control center's Battery module (design/26-DETAILS.md
//! 5.2.9): the battery ring with its percentage under it. The arc follows its level (linearly over
//! `--t-move`) and the number changes instantly (design/30 section 1.3).

use crate::components::content::text_runs::TextLine;
use crate::shell::battery::level::{
    BOLT, RingLayer, RingMark, RingTone, given, ring, use_ring_share,
};
use crate::shell::battery::ring::{RingSpan, arc_path};
use dioxus::prelude::*;
use ds_core::vocab::Fraction;
use ds_core::word::Word;

/// The ring at `level` (permille), charging or not, named `label`, with the percentage under it;
/// `children` (the device's glyph, optional) sit in the ring's middle.
#[component]
pub fn DeviceBattery(
    level: Fraction,
    #[props(default)] mark: RingMark,
    #[props(into)] label: TextLine,
    children: Element,
) -> Element {
    let level = level.clamped();
    let percent = level.whole_percent();
    let share = use_ring_share(level);
    let span = match mark {
        RingMark::Plain => RingSpan::FULL,
        RingMark::Charging => RingSpan::GAPPED,
    };
    rsx! {
        div { class: "ds-device-battery",
            div {
                class: "ds-battery",
                "data-tone": RingTone::of(level, mark).slug(),
                "data-mark": mark.attr(),
                role: "progressbar",
                "aria-label": "{label.plain_text()}",
                "aria-valuemin": "0",
                "aria-valuemax": "100",
                "aria-valuenow": "{percent}",
                {ring(RingLayer::Track, arc_path(span))}
                {ring(RingLayer::Arc, arc_path(span.filled(share)))}
                if let Some(device) = given(children) {
                    span { class: "ds-battery-device", {device} }
                }
                if mark == RingMark::Charging {
                    svg {
                        class: "ds-battery-bolt",
                        "data-ds-svg": "battery",
                        view_box: "0 0 10 16",
                        "aria-hidden": "true",
                        path { d: BOLT, fill: "currentColor" }
                    }
                }
            }
            span { class: "ds-battery-figure", "aria-hidden": "true", "{percent}%" }
        }
    }
}
