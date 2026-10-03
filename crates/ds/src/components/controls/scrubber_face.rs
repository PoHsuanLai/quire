//! The scrubber's drawing: the bar, its buffered bands, the played fill, the thumb and the
//! tooltip, in the pose and at the place the caller gives. `Scrubber` holds the pointer and
//! feeds this; a host that owns the pose itself (the gallery, a golden) draws it directly.
//!
//! Markup: `div.ds-scrubber[role=slider][data-state][data-size][data-availability]`, the played
//! share as `--f` and the tooltip's place as `--tip`; parts `track` (holding `buffered`, one per
//! band, and `fill`), `thumb`, and `tooltip` (shown only in the hover and dragging poses).

use crate::components::controls::scrubber_model::{BufferedRange, ScrubPose, merged, time_text};
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::vocab::{Availability, Fraction};
use ds_core::word::Word;
use ds_motion::spring::Millis;
use ds_style::tokens::control_size::ControlSize;

/// A recording's progress bar in `pose`, `position` of the way through `length`. The handlers
/// are the root element's own; `Scrubber` wires them to its machine.
#[component]
pub fn ScrubberFace(
    label: String,
    position: Fraction,
    length: Millis,
    #[props(default)] buffered: Vec<BufferedRange>,
    #[props(default)] pose: ScrubPose,
    #[props(default)] pointer: Fraction,
    #[props(default)] size: ControlSize,
    #[props(default)] availability: Availability,
    #[props(default)] onmounted: EventHandler<MountedEvent>,
    #[props(default)] onpointerenter: EventHandler<PointerEvent>,
    #[props(default)] onpointermove: EventHandler<PointerEvent>,
    #[props(default)] onpointerdown: EventHandler<PointerEvent>,
    #[props(default)] onpointerup: EventHandler<PointerEvent>,
    #[props(default)] onpointerleave: EventHandler<PointerEvent>,
    #[props(default)] onpointercancel: EventHandler<PointerEvent>,
    #[props(default)] onkeydown: EventHandler<KeyboardEvent>,
    #[props(default)] common: Common,
) -> Element {
    let position = position.clamped();
    let takes_input = availability == Availability::Enabled;
    let tooltip = match pose {
        ScrubPose::Idle => None,
        ScrubPose::Hover | ScrubPose::Dragging => {
            Some((time_text(length, pointer), pointer.clamped().css()))
        }
    };
    let class = common.class("ds-scrubber");
    let data = common.data_attributes();
    let fill = position.css();
    let tip = tooltip
        .as_ref()
        .map_or_else(|| "0".to_owned(), |(_, at)| at.clone());
    rsx! {
        div {
            id: common.id.clone(),
            class,
            role: "slider",
            tabindex: if takes_input { Some("0") } else { None },
            "data-state": pose.slug(),
            "data-size": size.slug(),
            "data-availability": availability.slug(),
            "data-wheel": "capture",
            "aria-label": common.aria_label.clone().unwrap_or_else(|| label.clone()),
            "aria-valuemin": "0",
            "aria-valuemax": "{length.0 / 1000}",
            "aria-valuenow": "{u64::from(length.0) * u64::from(position.0) / 1_000_000}",
            "aria-valuetext": "{time_text(length, position)}",
            "aria-disabled": availability.aria_disabled(),
            "aria-busy": availability.aria_busy(),
            style: "--f:{fill};--tip:{tip}",
            onmounted: move |event| {
                onmounted.call(event.clone());
                common.mounted(event);
            },
            onpointerenter: move |event| onpointerenter.call(event),
            onpointermove: move |event| onpointermove.call(event),
            onpointerdown: move |event| onpointerdown.call(event),
            onpointerup: move |event| onpointerup.call(event),
            onpointerleave: move |event| onpointerleave.call(event),
            onpointercancel: move |event| onpointercancel.call(event),
            onkeydown: move |event| onkeydown.call(event),
            ..data,
            div { class: "ds-scrubber-track",
                for band in merged(&buffered) {
                    div {
                        key: "{band.from.0}-{band.to.0}",
                        class: "ds-scrubber-buffered",
                        style: "--from:{band.from.css()};--to:{band.to.css()}",
                    }
                }
                div { class: "ds-scrubber-fill" }
            }
            div { class: "ds-scrubber-thumb" }
            if let Some((text, _)) = tooltip {
                div { class: "ds-scrubber-tooltip", "aria-hidden": "true", "{text}" }
            }
        }
    }
}
