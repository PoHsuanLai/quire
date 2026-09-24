//! The editor's rows below the field: the stop chips, the grain, the presets and the measured
//! contrast checks (design/04-COMPONENTS.md section 32).

use super::{DotIndex, Picker, dot_index, edit};
use crate::appearance::Scheme;
use crate::components::button::{Button, ButtonVariant};
use crate::components::chip::{Chip, ChipVariant};
use crate::components::section_header::{HeaderKind, SectionHeader};
use crate::components::slider::Slider;
use crate::components::vocab::Fraction;
use crate::icon::Icon;
use crate::icon::render::{Glyph, IconSize};
use crate::space::{Capping, Grain, PRESETS, SpaceLook, Verdict, derive, gradient, readout};
use dioxus::prelude::*;

/// The stop chips under the field and the "+ Colour" button.
#[component]
pub(super) fn Stops(
    look: SpaceLook,
    scheme: Scheme,
    current: usize,
    picker: Picker,
    onchange: EventHandler<SpaceLook>,
) -> Element {
    let palette = derive(&look.dots, scheme);
    let removable = look.dots.len() > 1;
    let room = look.dots.len() < edit::MAX_DOTS;
    let look_add = look.clone();
    rsx! {
        div { class: "ds-stops",
            for (index, dot) in look.dots.iter().copied().enumerate() {
                span {
                    key: "{index}",
                    class: "ds-stop",
                    "aria-pressed": if index == current { "true" } else { "false" },
                    onclick: move |_| picker.pick(dot_index(index)),
                    i {
                        class: "ds-stop-disc",
                        style: "background:{palette.picked.get(index).cloned().unwrap_or_default()}",
                    }
                    "{dot.hue.round()}°"
                    if removable {
                        button {
                            r#type: "button",
                            class: "ds-stop-remove",
                            "aria-label": "Remove colour",
                            onclick: {
                                let look = look.clone();
                                move |event: Event<MouseData>| {
                                    event.stop_propagation();
                                    picker.pick(DotIndex(0));
                                    onchange.call(edit::removed(&look, index));
                                }
                            },
                            Glyph { icon: Icon::X, size: IconSize::Micro }
                        }
                    }
                }
            }
            if room {
                Button {
                    variant: ButtonVariant::Mini,
                    label: "Colour",
                    icon: Icon::Plus,
                    onclick: move |_| {
                        let next = edit::added(&look_add);
                        picker.pick(dot_index(next.dots.len() - 1));
                        onchange.call(next);
                    },
                }
            }
        }
    }
}

/// The grain row: a Field header with the value, over a Slider (0-100 as thousandths).
#[component]
pub(super) fn GrainRow(look: SpaceLook, onchange: EventHandler<SpaceLook>) -> Element {
    let grain = look.grain.0.min(100);
    rsx! {
        div {
            SectionHeader { kind: HeaderKind::Field, text: "Grain", value: grain.to_string() }
            Slider {
                label: "Grain",
                value: Fraction(u16::from(grain) * 10),
                step: Fraction(10),
                onchange: move |fraction: Fraction| {
                    let grain = Grain(edit::grain_of(fraction));
                    onchange.call(SpaceLook { grain, ..look.clone() });
                },
            }
        }
    }
}

/// The eight presets, each painted with its own gradient in this scheme and named for itself.
#[component]
pub(super) fn Presets(
    look: SpaceLook,
    scheme: Scheme,
    picker: Picker,
    onchange: EventHandler<SpaceLook>,
) -> Element {
    rsx! {
        div {
            SectionHeader { kind: HeaderKind::Field, text: "Presets" }
            div { class: "ds-presets",
                for (index, preset) in PRESETS.iter().enumerate() {
                    button {
                        key: "{index}",
                        r#type: "button",
                        class: "ds-preset",
                        "aria-label": preset.name,
                        title: preset.name,
                        style: "background:{gradient(&derive(preset.dots, scheme))}",
                        onclick: {
                            let look = look.clone();
                            move |_| {
                                picker.pick(DotIndex(0));
                                onchange.call(edit::preset(&look, index));
                            }
                        },
                    }
                }
            }
        }
    }
}

/// The four measured pairs and the capping note under "Measured, this Space, this theme"
/// (design/03-COLOR.md section 6).
#[component]
pub(super) fn Checks(look: SpaceLook, scheme: Scheme) -> Element {
    rsx! {
        div {
            SectionHeader { kind: HeaderKind::Field, text: "Measured, this Space, this theme" }
            CheckRows { look, scheme }
        }
    }
}

/// The four measured pairs in `scheme` and its capping note.
#[component]
pub(super) fn CheckRows(look: SpaceLook, scheme: Scheme) -> Element {
    let checks = readout(&look, scheme);
    let note = match derive(&look.dots, scheme).capped {
        Capping::Uncapped => "No capping needed: every stop passes at the chroma you chose.",
        Capping::Capped => {
            "Chroma was lowered on at least one stop so the text above passes. Drop the dot lower on the field to see the uncapped colour."
        }
    };
    rsx! {
        div { class: "ds-checks",
            for check in checks {
                div { class: "ds-check",
                    span { "{check.label}" }
                    span { class: "ds-check-value", "{check.measured:.2}" }
                    Chip {
                        variant: ChipVariant::Status(check.verdict()),
                        text: match check.verdict() {
                            Verdict::Pass => format!("≥ {}", check.need),
                            Verdict::Fail => format!("< {}", check.need),
                        },
                    }
                }
            }
        }
        p { class: "ds-capnote", "{note}" }
    }
}
