//! The editor's hue x chroma field with its draggable handles: pointer down picks the handle
//! under it (or keeps the active one) and moves it, a drag follows, arrow keys nudge
//! (design/04-COMPONENTS.md section 32 behaviour).

use super::{Picker, dot_index, edit, field};
use crate::appearance::Scheme;
use crate::components::vocab::Switch;
use crate::geometry::measure::client_rect;
use crate::geometry::{Point, Px, Rect};
use crate::motion::drag::{DragPhase, use_drag};
use crate::space::dot_paint::DotPaint;
use crate::space::{SpaceLook, derive};
use dioxus::prelude::*;
use edit::Nudge;
use std::rc::Rc;

/// A handle's reach: pointer down within its radius picks it (`S:255`, 22 px across).
const HANDLE_RADIUS: f32 = 11.0;

/// The field follows the pointer from the first pixel (`S:1435-1441`).
const NO_THRESHOLD: Px = Px(0.0);

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

/// The hue x chroma plane and its handles.
#[component]
pub(super) fn Field(
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
                    let Some(rect) = client_rect(&mounted).await else { return };
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
            div { class: "ds-field-plane", style: "background-image:url({plane.colours})" }
            div { class: "ds-field-dots", style: "background-image:url({plane.dots})" }
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
            style: "left:{left};top:{top};{DotPaint::solid(&fill).style_attr()}",
            onkeydown: move |event| {
                if let Some(nudge) = Nudge::of(&event.key()) {
                    event.prevent_default();
                    onkey.call(nudge);
                }
            },
        }
    }
}
