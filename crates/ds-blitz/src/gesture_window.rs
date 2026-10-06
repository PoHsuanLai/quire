//! The window's side of the pinch gesture (`ds::host::gesture`): winit reports a touchpad pinch
//! with its phase and Blitz forwards none of it, so the window hook turns it into the vocabulary
//! a component listens to. A wheel's scroll is `window_scroll`'s: it moves the engine's
//! containers and tells the listeners itself.

use dioxus_native::winit::event::{TouchPhase, WindowEvent};
use ds::host::gesture::{Gesture, GesturePhase, Magnification};
use ds::prelude::*;

/// The pinch a winit event makes, with the pointer last seen at `at` (the window's logical
/// pixels).
pub(crate) fn pinch_of(event: &WindowEvent, at: Point) -> Option<Gesture> {
    match event {
        WindowEvent::PinchGesture { delta, phase, .. } => Some(Gesture::Pinch {
            phase: phase_of(*phase),
            by: Magnification::from_fraction(*delta),
            at,
        }),
        _ => None,
    }
}

pub(crate) fn phase_of(phase: TouchPhase) -> GesturePhase {
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
            pinch_of(&event, AT),
            Some(Gesture::Pinch {
                phase: GesturePhase::Changed,
                by: Magnification(50),
                at: AT,
            })
        );
    }

    #[test]
    fn other_events_are_not_pinches() {
        assert_eq!(pinch_of(&WindowEvent::Destroyed, AT), None);
    }
}
