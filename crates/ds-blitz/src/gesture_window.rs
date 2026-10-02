//! The window's side of the gestures (`ds::host::gesture`): winit reports a touchpad pinch and
//! a wheel with its phase, and Blitz forwards neither, so the window hook turns them into the
//! vocabulary a component listens to. The hook hears every winit event before the document.

use dioxus::prelude::Modifiers;
use dioxus_native::winit::event::{MouseScrollDelta, TouchPhase, WindowEvent};
use ds::host::gesture::{Gesture, GesturePhase, Magnification};
use ds::prelude::*;

/// How far one wheel detent moves the content: the 20 logical pixels Blitz scrolls a line by.
const LINE_PX: f64 = 20.0;

/// The gesture a winit event makes, with the pointer last seen at `at` (the window's logical
/// pixels), the window's display `scale` and the modifier keys `held`.
pub(crate) fn gesture_of(
    event: &WindowEvent,
    scale: f64,
    at: Point,
    held: Modifiers,
) -> Option<Gesture> {
    let scale = scale.max(f64::EPSILON);
    match event {
        WindowEvent::PinchGesture { delta, phase, .. } => Some(Gesture::Pinch {
            phase: phase_of(*phase),
            by: Magnification::from_fraction(*delta),
            at,
        }),
        WindowEvent::MouseWheel { delta, phase, .. } => {
            let (x, y) = match delta {
                MouseScrollDelta::LineDelta(x, y) => {
                    (f64::from(*x) * LINE_PX, f64::from(*y) * LINE_PX)
                }
                MouseScrollDelta::PixelDelta(pixels) => (pixels.x / scale, pixels.y / scale),
                _ => return None,
            };
            Some(Gesture::Scroll {
                phase: phase_of(*phase),
                by: Point {
                    x: Px(x as f32),
                    y: Px(y as f32),
                },
                at,
                held,
            })
        }
        _ => None,
    }
}

/// The pointer's place in logical pixels after a move, when `event` is one.
pub(crate) fn pointer_at(event: &WindowEvent, scale: f64) -> Option<Point> {
    let scale = scale.max(f64::EPSILON);
    match event {
        WindowEvent::PointerMoved { position, .. } => Some(Point {
            x: Px((position.x / scale) as f32),
            y: Px((position.y / scale) as f32),
        }),
        _ => None,
    }
}

fn phase_of(phase: TouchPhase) -> GesturePhase {
    match phase {
        TouchPhase::Started => GesturePhase::Began,
        TouchPhase::Moved => GesturePhase::Changed,
        TouchPhase::Ended => GesturePhase::Ended,
        TouchPhase::Cancelled => GesturePhase::Cancelled,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus_native::winit::dpi::PhysicalPosition;

    const AT: Point = Point {
        x: Px(10.0),
        y: Px(20.0),
    };

    #[test]
    fn a_pinch_keeps_its_phase_and_the_pointers_place() {
        let event = WindowEvent::PinchGesture {
            device_id: None,
            delta: 0.05,
            phase: TouchPhase::Moved,
        };
        assert_eq!(
            gesture_of(&event, 2.0, AT, Modifiers::empty()),
            Some(Gesture::Pinch {
                phase: GesturePhase::Changed,
                by: Magnification(50),
                at: AT,
            })
        );
    }

    #[test]
    fn a_wheel_is_logical_pixels_with_its_phase_and_the_modifiers_held() {
        // name, delta, scale, phase, expected movement
        let cases = [
            (
                "pixels at 2x",
                MouseScrollDelta::PixelDelta(PhysicalPosition::new(40.0, -80.0)),
                2.0,
                TouchPhase::Ended,
                (20.0, -40.0, GesturePhase::Ended),
            ),
            (
                "lines are 20 px",
                MouseScrollDelta::LineDelta(0.0, 3.0),
                2.0,
                TouchPhase::Started,
                (0.0, 60.0, GesturePhase::Began),
            ),
        ];
        for (name, delta, scale, phase, (x, y, want)) in cases {
            let event = WindowEvent::MouseWheel {
                device_id: None,
                delta,
                phase,
            };
            let got = gesture_of(&event, scale, AT, Modifiers::CONTROL);
            assert_eq!(
                got,
                Some(Gesture::Scroll {
                    phase: want,
                    by: Point { x: Px(x), y: Px(y) },
                    at: AT,
                    held: Modifiers::CONTROL,
                }),
                "{name}"
            );
        }
    }

    #[test]
    fn other_events_are_not_gestures() {
        assert_eq!(
            gesture_of(&WindowEvent::Destroyed, 1.0, AT, Modifiers::empty()),
            None
        );
    }
}
