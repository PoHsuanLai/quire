//! The mail app's Space editor rows: the name field, the Motion row and the readout per scheme
//! (both for a System Space, one for a Dark one), an unnamed Space's placeholder, and the Motion
//! row alone. Each is a golden under `lists/space_editor`.

use dioxus::prelude::*;
use ds::{CardAccent, Motion, PRESETS, Scheme, SpaceLook, Theme};
use ds_shell::{DotIndex, MeasuredIn, MotionChoice, SpaceEditor};

/// Preset `index` as a Space's look, in `theme`.
fn look(index: usize, theme: Theme) -> SpaceLook {
    SpaceLook {
        dots: PRESETS[index].dots.to_vec(),
        grain: ds::Grain(35),
        theme,
        card_accent: CardAccent::SpaceHue,
    }
}

/// The editor with every mail-app row switched on, for a Space whose theme is `theme`.
fn editor_rows(theme: Theme) -> Element {
    rsx! {
        SpaceEditor {
            look: look(0, theme),
            scheme: Scheme::Light,
            active_dot: DotIndex(0),
            onchange: |_| {},
            name: "Work".to_string(),
            on_rename: EventHandler::new(|_: String| {}),
            motion: MotionChoice { level: Motion::Reduced, on_motion: EventHandler::new(|_: Motion| {}) },
            measured: MeasuredIn::EachScheme,
        }
    }
}

/// One state and the golden it must match (relative to `tests/snapshots`).
pub struct Case {
    pub golden: &'static str,
    pub make: fn() -> Element,
}

pub const CASES: &[Case] = &[
    Case {
        golden: "lists/space_editor/rows-system.html",
        make: || editor_rows(Theme::System),
    },
    Case {
        golden: "lists/space_editor/rows-dark.html",
        make: || editor_rows(Theme::Dark),
    },
    Case {
        golden: "lists/space_editor/rename-unnamed.html",
        make: || rsx! { SpaceEditor { look: look(2, Theme::Light), scheme: Scheme::Light, active_dot: DotIndex(0), onchange: |_| {}, on_rename: EventHandler::new(|_: String| {}) } },
    },
    Case {
        golden: "lists/space_editor/motion-row.html",
        make: || {
            rsx! {
                SpaceEditor {
                    look: SpaceLook::default(),
                    scheme: Scheme::Light,
                    active_dot: DotIndex(0),
                    onchange: |_| {},
                    motion: MotionChoice { level: Motion::Reduced, on_motion: EventHandler::new(|_: Motion| {}) },
                }
            }
        },
    },
];
