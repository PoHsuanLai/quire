//! Space: the Space editor bound to the gallery's own Space, the frame tokens it derives, its
//! contrast readout, and the eight presets as Space dots.

use super::{Caption, Section};
use crate::axes::{Axes, PresetIndex};
use dioxus::prelude::*;
use ds::{
    Chip, ChipVariant, DotIndex, FrameVars, Here, Key, MeasuredIn, MotionChoice, Shortcut,
    SpaceDot, SpaceEditor, readout, use_env,
};

/// The Space page.
#[component]
pub fn SpacePage() -> Element {
    let mut axes = use_context::<Signal<Axes>>();
    let scheme = use_env().scheme;
    let (look, preset) = {
        let axes = axes.read();
        (axes.look.clone(), axes.preset)
    };
    let frame = FrameVars::of(&look, scheme);
    let mut renamed = use_signal(|| None::<String>);
    let name = renamed().unwrap_or_else(|| preset.label());
    let motion = axes.read().motion;
    let checks = readout(&look, scheme);
    rsx! {
        Section { title: "Presets", note: "The eight presets as the sidebar foot draws them. Picking one resets the Space; the toolbar's Space menu does the same.",
            div { class: "g-row",
                for (index , candidate) in PresetIndex::all().enumerate() {
                    SpaceDot {
                        key: "{index}",
                        name: candidate.label(),
                        frame: FrameVars::of(&candidate.look(), scheme),
                        here: if candidate == preset { Here::Current } else { Here::Elsewhere },
                        shortcut: Shortcut(vec![Key::Ctrl, Key::Char(char::from(b'1' + candidate.0))]),
                        onclick: move |_| axes.with_mut(|axes| *axes = axes.clone().with_preset(candidate)),
                    }
                }
            }
        }
        Section { title: "SpaceEditor", note: "Edits the Space the whole gallery is framed in: drag a dot, change the grain, pick a preset. Rename it in the title; the Motion row drives the gallery's motion; the readout measures each scheme the Space's theme can show.",
            SpaceEditor {
                look: look.clone(),
                scheme,
                active_dot: DotIndex(0),
                name: Some(name),
                onchange: move |next| axes.with_mut(|axes| axes.look = next),
                on_rename: move |next: String| renamed.set(Some(next)),
                motion: MotionChoice { level: motion, on_motion: EventHandler::new(move |next| axes.with_mut(|axes| axes.motion = next)) },
                measured: MeasuredIn::EachScheme,
            }
        }
        Section { title: "Contrast readout", note: "ds::readout for this Space in the current scheme: the four gates every pick is held to.",
            div { class: "g-row",
                for check in checks {
                    Chip {
                        variant: ChipVariant::Status(check.verdict()),
                        text: format!("{}: {:.2} (needs {})", check.label, check.measured, check.need),
                    }
                }
            }
        }
        Section { title: "Frame tokens", note: "FrameVars::of(look, scheme): what the root writes inline.",
            div { class: "g-grid3",
                Caption { name: "--f-ink", code: frame.ink.clone() }
                Caption { name: "--f-ink-soft", code: frame.ink_soft.clone() }
                Caption { name: "--f-ink-faint", code: frame.ink_faint.clone() }
                Caption { name: "--f-pill", code: frame.pill.clone() }
                Caption { name: "--f-pill-hover", code: frame.pill_hover.clone() }
                Caption { name: "--f-line", code: frame.line.clone() }
                Caption { name: "--f-solid", code: frame.solid.clone() }
                Caption { name: "--f-grain", code: frame.grain_opacity.clone() }
                Caption {
                    name: "card accent",
                    code: frame.accent.as_ref().map_or("Postmark (the card keeps its own)".to_string(), |[accent, soft, ink]| format!("{accent} / {soft} / {ink}")),
                }
            }
            Caption { name: "--f-grad", code: frame.gradient.clone() }
            div { class: "g-row",
                for (name , var) in [("ink", "--f-ink"), ("soft", "--f-ink-soft"), ("faint", "--f-ink-faint"), ("pill", "--f-pill"), ("hover", "--f-pill-hover"), ("solid", "--f-solid")] {
                    div { class: "g-col",
                        div { class: "g-chip-swatch", style: "background:var({var})" }
                        span { class: "g-code", "{name}" }
                    }
                }
            }
        }
    }
}
