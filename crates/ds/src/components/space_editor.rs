//! SpaceEditor: the field, dots, stops, grain, presets and contrast checks, plus the Space dots
//! that switch Spaces (design/04-COMPONENTS.md section 32, design/21-SPACES.md section 6).
//! Every colour it shows comes from `space::palette`; it computes none.

mod edit;
mod field;
mod parts;
mod png;

use crate::appearance::{Scheme, Theme};
use crate::components::section_header::{HeaderKind, SectionHeader};
use crate::components::segmented::SegmentedControl;
use crate::components::vocab::{Here, Shortcut, Switch};
use crate::geometry::{Point, Px, Rect};
use crate::motion::drag::{DragPhase, use_drag};
use crate::space::{CardAccent, FrameVars, SpaceLook, derive, gradient};
use dioxus::prelude::*;
use edit::Nudge;
use parts::{Checks, GrainRow, Presets, Stops};
use std::rc::Rc;

/// Which of a Space's dots is being edited: 0, 1 or 2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct DotIndex(pub u8);

/// A handle's reach: pointer down within its radius picks it (`S:255`, 22 px across).
const HANDLE_RADIUS: f32 = 11.0;

/// The field follows the pointer from the first pixel (`S:1435-1441`).
const NO_THRESHOLD: Px = Px(0.0);

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

/// The handle under `at` on a field occupying `rect`, if any.
fn handle_under(look: &SpaceLook, rect: Rect, at: Point) -> Option<usize> {
    look.dots.iter().position(|dot| {
        let (x, y) = field::place(*dot);
        let cx = rect.left().0 + x as f32 * rect.size.width.0;
        let cy = rect.top().0 + y as f32 * rect.size.height.0;
        (at.x.0 - cx).hypot(at.y.0 - cy) <= HANDLE_RADIUS
    })
}

/// The pointer's place on the field as fractions across and down.
fn fractions(rect: Rect, at: Point) -> (f64, f64) {
    let share = |offset: f32, length: f32| {
        if length > 0.0 {
            f64::from(offset / length)
        } else {
            0.0
        }
    };
    (
        share(at.x.0 - rect.left().0, rect.size.width.0),
        share(at.y.0 - rect.top().0, rect.size.height.0),
    )
}

/// A dioxus client point as a layout point.
fn point(at: dioxus::html::geometry::ClientPoint) -> Point {
    Point {
        x: Px(at.x as f32),
        y: Px(at.y as f32),
    }
}

/// A percentage for an inline `left` or `top`.
fn percent(share: f64) -> String {
    format!("{:.2}%", share * 100.0)
}

/// The Space editor panel.
///
/// Controlled: every edit is emitted as a whole new [`SpaceLook`] through `onchange`, and the
/// consumer passes it back. `active_dot` seeds which dot the handles and keys move; picking a
/// handle or a stop inside the editor moves it until the consumer passes a different one, and
/// reports it through `on_active_dot` so the consumer can pass it back. `name` titles the panel
/// "{name} Space" (section 32 markup); without one it reads "Space".
#[component]
pub fn SpaceEditor(
    look: SpaceLook,
    scheme: Scheme,
    active_dot: DotIndex,
    onchange: EventHandler<SpaceLook>,
    #[props(default)] name: Option<String>,
    #[props(default)] on_active_dot: Option<EventHandler<ActiveDot>>,
) -> Element {
    let picked = use_signal(|| None::<(DotIndex, DotIndex)>);
    let picker = Picker {
        picked,
        prop: active_dot,
        report: on_active_dot,
    };
    let title = match name {
        Some(name) => format!("{name} Space"),
        None => "Space".to_string(),
    };
    let dots = look.dots.len();
    let current = active(active_dot, picked(), dots);
    let palette = derive(&look.dots, scheme);
    let swatch = gradient(&palette);
    rsx! {
        aside { class: "ds-space-editor", "aria-label": "Space editor",
            h3 { class: "ds-space-editor-title",
                span { class: "ds-space-swatch", style: "background:{swatch}" }
                "{title}"
            }
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
            Checks { look, scheme }
        }
    }
}

/// The hue x chroma plane and its handles.
#[component]
fn Field(
    look: SpaceLook,
    scheme: Scheme,
    current: usize,
    picker: Picker,
    onchange: EventHandler<SpaceLook>,
) -> Element {
    let drag = use_drag::<usize>(NO_THRESHOLD);
    let mut element = use_signal(|| None::<Rc<MountedData>>);
    let mut bounds = use_signal(|| None::<Rect>);
    let palette = derive(&look.dots, scheme);
    let plane = field::plane(scheme);
    let look_down = look.clone();
    let look_move = look.clone();
    rsx! {
        div {
            class: "ds-field",
            onmounted: move |event| element.set(Some(event.data())),
            onpointerdown: move |event| {
                let at = point(event.client_coordinates());
                let look = look_down.clone();
                let Some(mounted) = element() else { return };
                spawn(async move {
                    let Ok(measured) = mounted.get_client_rect().await else { return };
                    let rect = super::hover_strip::rect(measured);
                    bounds.set(Some(rect));
                    let index = handle_under(&look, rect, at).unwrap_or(current);
                    picker.pick(dot_index(index));
                    drag.down(index, at);
                    let (x, y) = fractions(rect, at);
                    onchange.call(edit::moved(&look, index, field::dot_at(x, y)));
                });
            },
            onpointermove: move |event| {
                let (DragPhase::Pending { key, .. } | DragPhase::Live { key, .. }) = drag.phase() else {
                    return;
                };
                let at = point(event.client_coordinates());
                drag.moved(at);
                if let Some(rect) = bounds() {
                    let (x, y) = fractions(rect, at);
                    onchange.call(edit::moved(&look_move, key, field::dot_at(x, y)));
                }
            },
            onpointerup: move |_| {
                drag.up();
            },
            div { class: "ds-field-plane", style: "background-image:url({plane})" }
            for (index, dot) in look.dots.iter().copied().enumerate() {
                Handle {
                    key: "{index}",
                    index,
                    left: percent(field::place(dot).0),
                    top: percent(field::place(dot).1),
                    fill: palette.picked.get(index).cloned().unwrap_or_default(),
                    text: format!("{}°, {}%", dot.hue.round(), (dot.chroma * 100.0).round()),
                    on: if index == current { Switch::On } else { Switch::Off },
                    onkey: {
                        let look = look.clone();
                        move |nudge: Nudge| {
                            picker.pick(dot_index(index));
                            onchange.call(edit::nudged(&look, index, nudge));
                        }
                    },
                }
            }
        }
    }
}

/// One draggable dot: a slider over hue and chroma, moved by arrow keys.
#[component]
fn Handle(
    index: usize,
    left: String,
    top: String,
    fill: String,
    text: String,
    on: Switch,
    onkey: EventHandler<Nudge>,
) -> Element {
    let number = index + 1;
    rsx! {
        div {
            class: "ds-handle",
            role: "slider",
            tabindex: "0",
            "aria-label": "Colour {number}",
            "aria-valuetext": "{text}",
            "aria-pressed": on.aria(),
            style: "left:{left};top:{top};background:{fill}",
            onkeydown: move |event| {
                if let Some(nudge) = Nudge::of(&event.key()) {
                    event.prevent_default();
                    onkey.call(nudge);
                }
            },
        }
    }
}

/// One Space's dot in the sidebar foot.
#[component]
pub fn SpaceDot(
    name: String,
    frame: FrameVars,
    here: Here,
    shortcut: Shortcut,
    onclick: EventHandler<()>,
) -> Element {
    let pressed = match here {
        Here::Current => Switch::On,
        Here::Elsewhere => Switch::Off,
    };
    let keys = shortcut.glyphs();
    rsx! {
        button {
            r#type: "button",
            class: "ds-space-dot",
            "aria-pressed": pressed.aria(),
            "aria-label": "{name} Space",
            title: "{name} ({keys})",
            style: "background:{frame.gradient}",
            onclick: move |_| onclick.call(()),
        }
    }
}
