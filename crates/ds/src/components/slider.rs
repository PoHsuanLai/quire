//! Slider: a continuous value, divs and a drag tracker, because Blitz has no native range
//! (design/04-COMPONENTS.md section 5).

use crate::components::vocab::{Availability, Fraction};
use crate::geometry::measure::client_rect;
use crate::geometry::units::{Point, Px, Rect};
use crate::motion::drag::{DragPhase, use_drag};
use dioxus::html::geometry::ClientPoint;
use dioxus::prelude::*;
use std::rc::Rc;

/// The keyboard step when the consumer leaves `step` at its default of zero: the editor
/// handle's `.05` (`S:1455-1456`), the step design/04-COMPONENTS.md cites for keys.
const DEFAULT_STEP: Fraction = Fraction(50);

/// The slider moves as soon as the pointer goes down (`S:1435-1441`), so the tracker needs no
/// travel before it counts as a drag.
const NO_THRESHOLD: Px = Px(0.0);

/// One keyboard step: the prop, or [`DEFAULT_STEP`] when it is zero.
fn keyboard_step(step: Fraction) -> Fraction {
    match step.clamped() {
        Fraction(0) => DEFAULT_STEP,
        step => step,
    }
}

/// Which way a key moves the value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Nudge {
    Down,
    Up,
}

/// The key's nudge: Left and Down lower the value, Right and Up raise it.
fn nudge(key: &Key) -> Option<Nudge> {
    match key {
        Key::ArrowLeft | Key::ArrowDown => Some(Nudge::Down),
        Key::ArrowRight | Key::ArrowUp => Some(Nudge::Up),
        _ => None,
    }
}

/// `value` moved one `step`, held inside 0..=1000.
fn stepped(value: Fraction, step: Fraction, nudge: Nudge) -> Fraction {
    let value = value.clamped().0;
    match nudge {
        Nudge::Down => Fraction(value.saturating_sub(step.0)),
        Nudge::Up => Fraction(value.saturating_add(step.0)).clamped(),
    }
}

/// The value under the pointer at `x` on a slider occupying `rect`, clamped to its ends.
fn fraction_at(rect: Rect, x: Px) -> Fraction {
    let width = rect.size.width.0;
    if width <= 0.0 {
        return Fraction(0);
    }
    let share = ((x.0 - rect.left().0) / width).clamp(0.0, 1.0);
    // In 0..=1000 after the clamp, so the cast cannot truncate.
    Fraction((share * 1000.0).round() as u16)
}

/// `aria-valuenow`: the value on the 0..=100 scale the markup declares.
fn percent(value: Fraction) -> u16 {
    (value.clamped().0 + 5) / 10
}

/// A dioxus client point as a layout point.
fn point(at: ClientPoint) -> Point {
    Point {
        x: Px(at.x as f32),
        y: Px(at.y as f32),
    }
}

/// A value in a range.
///
/// The `DragTracker` owns the gesture: pointer down jumps the value to the pointer and marks
/// the thumb live, moves follow 1:1, up releases. The slider's rect is read in the pointer-down
/// handler, after layout, never in `onmounted` (spike S9), and the pointer-down also focuses the
/// slider, since Blitz focuses only text inputs on click (spike S12).
#[component]
pub fn Slider(
    label: String,
    value: Fraction,
    #[props(default)] step: Fraction,
    #[props(default)] availability: Availability,
    onchange: EventHandler<Fraction>,
) -> Element {
    let drag = use_drag::<()>(NO_THRESHOLD);
    let mut element = use_signal(|| None::<Rc<MountedData>>);
    let mut track = use_signal(|| None::<Rect>);
    let value = value.clamped();
    let step = keyboard_step(step);
    let fill = value.css();
    let now = percent(value);
    let thumb = match drag.phase() {
        DragPhase::Idle => "idle",
        DragPhase::Pending { .. } | DragPhase::Live { .. } => "live",
    };
    let enabled = availability == Availability::Enabled;
    rsx! {
        div {
            class: "ds-slider",
            role: "slider",
            tabindex: "0",
            "aria-label": "{label}",
            "aria-valuemin": "0",
            "aria-valuemax": "100",
            "aria-valuenow": "{now}",
            "aria-disabled": availability.aria_disabled(),
            style: "--f:{fill}",
            onmounted: move |event| element.set(Some(event.data())),
            onpointerdown: move |event| {
                if !enabled {
                    return;
                }
                let at = point(event.client_coordinates());
                drag.down((), at);
                if let Some(mounted) = element() {
                    spawn(async move {
                        // Focus is best-effort: a renderer without it still slides.
                        let _ = crate::focus::host::focus_element(&mounted).await;
                        if let Some(measured) = client_rect(&mounted).await {
                            track.set(Some(measured));
                            onchange.call(fraction_at(measured, at.x));
                        }
                    });
                }
            },
            onpointermove: move |event| {
                if drag.phase() == DragPhase::Idle {
                    return;
                }
                let at = point(event.client_coordinates());
                drag.moved(at);
                if let Some(measured) = track() {
                    onchange.call(fraction_at(measured, at.x));
                }
            },
            onpointerup: move |_| {
                drag.up();
            },
            onkeydown: move |event| {
                if let (true, Some(nudge)) = (enabled, nudge(&event.key())) {
                    onchange.call(stepped(value, step, nudge));
                }
            },
            div { class: "ds-slider-track",
                div { class: "ds-slider-fill" }
            }
            div { class: "ds-slider-thumb", "data-drag": thumb }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::units::Size;

    fn track(left: f32, width: f32) -> Rect {
        Rect {
            origin: Point {
                x: Px(left),
                y: Px(0.0),
            },
            size: Size {
                width: Px(width),
                height: Px(22.0),
            },
        }
    }

    #[test]
    fn the_pointer_sets_the_value_clamped_to_the_ends() {
        const CASES: &[(f32, u16)] = &[
            (100.0, 0),
            (150.0, 250),
            (200.0, 500),
            (300.0, 1000),
            (40.0, 0),
            (900.0, 1000),
        ];
        for (x, want) in CASES {
            assert_eq!(
                fraction_at(track(100.0, 200.0), Px(*x)),
                Fraction(*want),
                "x {x}"
            );
        }
        assert_eq!(
            fraction_at(track(100.0, 0.0), Px(150.0)),
            Fraction(0),
            "zero width"
        );
    }

    #[test]
    fn keys_step_and_stop_at_the_ends() {
        const CASES: &[(u16, u16, Nudge, u16)] = &[
            (500, 50, Nudge::Up, 550),
            (500, 50, Nudge::Down, 450),
            (980, 50, Nudge::Up, 1000),
            (20, 50, Nudge::Down, 0),
            (1400, 50, Nudge::Down, 950),
        ];
        for (value, step, way, want) in CASES {
            let got = stepped(Fraction(*value), Fraction(*step), *way);
            assert_eq!(got, Fraction(*want), "{value} {way:?} by {step}");
        }
    }

    #[test]
    fn a_zero_step_falls_back_to_the_handle_step() {
        assert_eq!(keyboard_step(Fraction(0)), DEFAULT_STEP);
        assert_eq!(keyboard_step(Fraction(10)), Fraction(10));
    }

    #[test]
    fn arrows_nudge_and_other_keys_do_not() {
        assert_eq!(nudge(&Key::ArrowLeft), Some(Nudge::Down));
        assert_eq!(nudge(&Key::ArrowDown), Some(Nudge::Down));
        assert_eq!(nudge(&Key::ArrowRight), Some(Nudge::Up));
        assert_eq!(nudge(&Key::ArrowUp), Some(Nudge::Up));
        assert_eq!(nudge(&Key::Enter), None);
    }

    #[test]
    fn valuenow_is_a_percent() {
        assert_eq!(percent(Fraction(350)), 35);
        assert_eq!(percent(Fraction(1000)), 100);
        assert_eq!(percent(Fraction(4000)), 100);
    }
}
