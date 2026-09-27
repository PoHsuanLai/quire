//! Slider: a continuous value, divs and a drag tracker, because Blitz has no native range
//! (design/04-COMPONENTS.md section 5).
//!
//! Driven motion (design/05 section 14, wave H1): the thumb is drawn from a spring. A drag moves
//! it 1:1; a release with speed throws the value to where the throw projects (`Throw`, about half
//! a second of its speed on), and the thumb springs there carrying the hand's velocity; a key or a
//! value from elsewhere springs it critically from where it is.

use crate::components::track::fraction_at;
use crate::components::vocab::{Availability, Fraction};
use crate::detail::{Contact, Touch};
use crate::geometry::measure::client_rect;
use crate::geometry::units::{Point, Px, Rect};
use crate::motion::drag::{DragPhase, use_drag};
use crate::motion::{
    PxPerUnit, SpringMotion, SpringResponse, SpringSpec, Throw, Velocity, VelocityMeter,
    use_spring_motion,
};
use dioxus::core::queue_effect;
use dioxus::html::geometry::ClientPoint;
use dioxus::prelude::*;
use std::rc::Rc;

/// A release slower than this, in pixels per second, sets the value where the thumb was let go;
/// faster, it throws (the swipe's fling speed, `notifications.swipe_dismiss_velocity_px_s`).
const THROW_PX_PER_S: i32 = 600;

/// The track's width before it has been measured: the thumb's units are thousandths of it.
const UNMEASURED_WIDTH: f32 = 200.0;

/// Where a release at `value` moving at `velocity` over a track `width` wide lands: the
/// projection, held to the ends, or `None` when it was too slow to throw.
fn thrown_to(value: Fraction, velocity: Velocity, width: f32) -> Option<Fraction> {
    if velocity.0.abs() < THROW_PX_PER_S || width <= 0.0 {
        return None;
    }
    let from = Px(f32::from(value.clamped().0) * width / 1000.0);
    let landed = Throw { from, velocity }.projected().0 / width * 1000.0;
    Some(Fraction(landed.clamp(0.0, 1000.0).round() as u16))
}

/// The thumb's spring for this render: it follows `value`, 1:1 while a drag holds it.
fn follow(
    knob: SpringMotion,
    mut seen: CopyValue<Fraction>,
    value: Fraction,
    phase: DragPhase<()>,
) {
    if *seen.peek() == value {
        return;
    }
    seen.set(value);
    let at = f32::from(value.0);
    match phase {
        DragPhase::Idle => queue_effect(move || {
            knob.go(
                at,
                SpringSpec::for_touch(Touch::Remote).response(SpringResponse::Quick),
            )
        }),
        DragPhase::Pending { .. } | DragPhase::Live { .. } => queue_effect(move || knob.track(at)),
    }
}

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
    let mut meter = use_signal(VelocityMeter::default);
    let value = value.clamped();
    let step = keyboard_step(step);
    let width = track
        .peek()
        .map_or(UNMEASURED_WIDTH, |rect| rect.size.width.0);
    let knob = use_spring_motion(f32::from(value.0), PxPerUnit(width / 1000.0));
    let mut seen = use_hook(|| CopyValue::new(value));
    follow(knob, seen, value, drag.phase());
    let shown = knob.frame().position().clamp(0.0, 1000.0);
    let fill = Fraction(shown.round() as u16).css();
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
            "data-wheel": "capture",
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
                meter.set(VelocityMeter::default().moved(at.x, crate::time::now()));
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
                let measured_now = meter.peek().moved(at.x, crate::time::now());
                meter.set(measured_now);
                if let Some(measured) = track() {
                    onchange.call(fraction_at(measured, at.x));
                }
            },
            onpointerup: move |event| {
                drag.up();
                let velocity = meter.peek().released(crate::time::now());
                meter.set(VelocityMeter::default());
                let width = track.peek().map_or(0.0, |rect| rect.size.width.0);
                if let (true, Some(to)) = (enabled, thrown_to(value, velocity, width)) {
                    let contact = Contact::from_event(&event).with_velocity(velocity);
                    let spec = SpringSpec::for_touch(Touch::Contact(contact))
                        .response(SpringResponse::Quick);
                    knob.go(f32::from(to.0), spec);
                    seen.set(to);
                    onchange.call(to);
                }
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
    fn a_fast_release_throws_the_value_and_a_slow_one_leaves_it() {
        const CASES: &[(u16, i32, f32, Option<u16>)] = &[
            (300, 0, 200.0, None),
            (300, 400, 200.0, None),
            (300, 800, 400.0, Some(1000)),
            (300, 700, 1000.0, Some(649)),
            (500, -1500, 200.0, Some(0)),
            (500, 900, 0.0, None),
        ];
        for &(value, v, width, want) in CASES {
            let got = thrown_to(Fraction(value), Velocity(v), width).map(|f| f.0);
            assert_eq!(got, want, "{value} at {v} px/s over {width}");
        }
    }

    #[test]
    fn valuenow_is_a_percent() {
        assert_eq!(percent(Fraction(350)), 35);
        assert_eq!(percent(Fraction(1000)), 100);
        assert_eq!(percent(Fraction(4000)), 100);
    }
}
