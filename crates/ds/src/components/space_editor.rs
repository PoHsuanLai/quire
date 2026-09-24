//! SpaceEditor: the field, dots, stops, grain, presets and contrast checks, plus the Space dots
//! that switch Spaces (design/04-COMPONENTS.md section 32, design/21-SPACES.md section 6).
//! Every colour it shows comes from `space::palette`; it computes none.

mod dot;
mod edit;
mod field;
mod handles;
mod parts;
pub(crate) mod png;
mod rows;

use crate::appearance::{Scheme, Theme};
use crate::components::section_header::{HeaderKind, SectionHeader};
use crate::components::segmented::SegmentedControl;
use crate::space::{CardAccent, SpaceLook};
use dioxus::prelude::*;
pub use dot::SpaceDot;
use handles::Field;
use parts::{Checks, GrainRow, Presets, Stops};
use rows::{EachScheme, MotionRow, Title};
pub use rows::{MeasuredIn, MotionChoice, MotionLevels};

/// Which of a Space's dots is being edited: 0, 1 or 2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct DotIndex(pub u8);

/// The dot being edited: the one picked inside the editor for the current `active_dot` prop, or
/// the prop itself, held inside the Space's dots.
fn active(prop: DotIndex, picked: Option<(DotIndex, DotIndex)>, dots: usize) -> usize {
    let chosen = match picked {
        Some((seen, chosen)) if seen == prop => chosen,
        _ => prop,
    };
    usize::from(chosen.0).min(dots.saturating_sub(1))
}

/// The dot the editor made active, as `on_active_dot` reports it: a handle or stop was picked,
/// a colour was added or removed, or a preset was applied.
pub type ActiveDot = DotIndex;

/// Picking a dot inside the editor: remembers the pick against the `active_dot` prop it was made
/// under, and reports it to the consumer.
#[derive(Clone, Copy, PartialEq)]
struct Picker {
    picked: Signal<Option<(DotIndex, DotIndex)>>,
    prop: DotIndex,
    report: Option<EventHandler<ActiveDot>>,
}

impl Picker {
    /// Make `index` the dot being edited.
    fn pick(self, index: DotIndex) {
        let mut picked = self.picked;
        picked.set(Some((self.prop, index)));
        if let Some(report) = self.report {
            report.call(index);
        }
    }
}

/// The dot at `index`, as a `DotIndex`.
fn dot_index(index: usize) -> DotIndex {
    DotIndex(u8::try_from(index).unwrap_or(u8::MAX))
}

/// The Space editor panel.
///
/// Controlled: every edit is emitted as a whole new [`SpaceLook`] through `onchange`, and the
/// consumer passes it back. `active_dot` seeds which dot the handles and keys move; picking a
/// handle or a stop inside the editor moves it until the consumer passes a different one, and
/// reports it through `on_active_dot` so the consumer can pass it back. `name` titles the panel
/// "{name} Space" (section 32 markup); without one it reads "Space".
///
/// Three rows are the consumer's to switch on, each absent by default: `on_rename` makes the
/// title an inline field holding `name`, each keystroke reported; `motion` adds a Motion row
/// (the Space's own motion, which the consumer feeds to its root's `appearance.motion`);
/// `measured: MeasuredIn::EachScheme` measures the contrast in each scheme the Space's theme
/// can show, each under its own heading, where the default measures the scheme it is drawn in.
/// `motion_levels` picks what the Motion row offers: every choice (the default), or
/// `MotionLevels::Contact`'s Calm, Standard and Extra, a Space's own three.
#[component]
pub fn SpaceEditor(
    look: SpaceLook,
    scheme: Scheme,
    active_dot: DotIndex,
    onchange: EventHandler<SpaceLook>,
    #[props(default)] name: Option<String>,
    #[props(default)] on_active_dot: Option<EventHandler<ActiveDot>>,
    #[props(default)] on_rename: Option<EventHandler<String>>,
    #[props(default)] motion: Option<MotionChoice>,
    #[props(default)] measured: MeasuredIn,
    #[props(default)] motion_levels: MotionLevels,
) -> Element {
    let picked = use_signal(|| None::<(DotIndex, DotIndex)>);
    let picker = Picker {
        picked,
        prop: active_dot,
        report: on_active_dot,
    };
    let dots = look.dots.len();
    let current = active(active_dot, picked(), dots);
    rsx! {
        aside { class: "ds-space-editor", "aria-label": "Space editor",
            Title { dots: look.dots.clone(), scheme, name, on_rename }
            div {
                SectionHeader { kind: HeaderKind::Field, text: "Colour", value: "drag a dot".to_string() }
                Field { look: look.clone(), scheme, current, picker, onchange }
                Stops { look: look.clone(), scheme, current, picker, onchange }
            }
            GrainRow { look: look.clone(), onchange }
            div {
                SectionHeader { kind: HeaderKind::Field, text: "Appearance" }
                SegmentedControl::<Theme> {
                    label: "Appearance",
                    options: Theme::ALL.into_iter().map(|theme| (theme, theme.label().to_string())).collect::<Vec<_>>(),
                    value: look.theme,
                    onchange: {
                        let look = look.clone();
                        move |theme| onchange.call(SpaceLook { theme, ..look.clone() })
                    },
                }
            }
            if let Some(choice) = motion {
                MotionRow { choice, levels: motion_levels }
            }
            div {
                SectionHeader { kind: HeaderKind::Field, text: "Accent inside the card" }
                SegmentedControl::<CardAccent> {
                    label: "Accent",
                    options: vec![
                        (CardAccent::SpaceHue, "A hint of the Space".to_string()),
                        (CardAccent::Postmark, "Postmark".to_string()),
                    ],
                    value: look.card_accent,
                    onchange: {
                        let look = look.clone();
                        move |card_accent| onchange.call(SpaceLook { card_accent, ..look.clone() })
                    },
                }
            }
            Presets { look: look.clone(), scheme, picker, onchange }
            match measured {
                MeasuredIn::ThisScheme => rsx! { Checks { look, scheme } },
                MeasuredIn::EachScheme => rsx! { EachScheme { look } },
            }
        }
    }
}
