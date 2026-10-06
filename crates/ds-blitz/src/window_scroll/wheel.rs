//! A wheel or touchpad event as the scroll engine and the gesture listeners take it: winit's
//! `MouseWheel` reduced to what it means. Pure.

use blitz_kit::scroll::engine::Kinetic;
use blitz_kit::scroll::geom::ScrollAxis;
use blitz_kit::scroll::pad::PointerScroll;
use ds::host::gesture::{Gesture, GesturePhase};
use ds::prelude::*;
use keyboard_types::Modifiers;

/// How far a wheel or touchpad turned, in winit's sign: positive means the content moves right
/// and down (the opposite of an offset).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WheelDelta {
    /// Wheel clicks, a detent each.
    Lines { x: f64, y: f64 },
    /// Logical pixels, a touchpad's own motion.
    Pixels { x: f64, y: f64 },
}

/// One wheel or touchpad event.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WheelInput {
    pub delta: WheelDelta,
    /// Where a touchpad gesture is; a wheel reports `Changed`.
    pub phase: GesturePhase,
}

/// What the engine does for `input` with `held` modifiers: wheel clicks become detents (Shift
/// turns a vertical wheel horizontal), a touchpad's pixels become a phased pan that ends with the
/// fingers' lift.
pub fn pointer_scrolls(input: WheelInput, held: Modifiers) -> Vec<PointerScroll> {
    match input.delta {
        WheelDelta::Lines { x, y } => detents(-x, -y, held),
        WheelDelta::Pixels { x, y } => pan(-x, -y, input.phase),
    }
}

/// The detents of a wheel turned `(sx, sy)` clicks in offset space.
fn detents(sx: f64, sy: f64, held: Modifiers) -> Vec<PointerScroll> {
    let shifted = held.contains(Modifiers::SHIFT);
    let detent = |axis, steps| vec![PointerScroll::Detents { axis, steps }];
    match (shifted, sy != 0.0) {
        (true, true) => detent(ScrollAxis::X, sy),
        _ if sx != 0.0 => detent(ScrollAxis::X, sx),
        (_, true) => detent(ScrollAxis::Y, sy),
        _ => Vec::new(),
    }
}

/// A touchpad's motion `(dx, dy)` in offset space, then its end if the fingers lifted.
fn pan(dx: f64, dy: f64, phase: GesturePhase) -> Vec<PointerScroll> {
    let moved = (dx != 0.0 || dy != 0.0).then_some(PointerScroll::Pan {
        dx,
        dy,
        kinetic: Kinetic::Momentum,
    });
    let lifted = match phase {
        GesturePhase::Ended | GesturePhase::Cancelled => Some(PointerScroll::Stop),
        GesturePhase::Began | GesturePhase::Changed => None,
    };
    moved.into_iter().chain(lifted).collect()
}

/// Whether a wheel event is clicks, which listeners may want eased, or a touchpad's motion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Wheel,
    Touchpad,
}

impl WheelInput {
    /// Which device this event is from.
    pub fn source(&self) -> Source {
        match self.delta {
            WheelDelta::Lines { .. } => Source::Wheel,
            WheelDelta::Pixels { .. } => Source::Touchpad,
        }
    }

    /// How far the content moves, in winit's sign: a click is `detent_px`, a touchpad's pixels
    /// are their own.
    pub fn motion(&self, detent_px: f64) -> (f64, f64) {
        match self.delta {
            WheelDelta::Lines { x, y } => (x * detent_px, y * detent_px),
            WheelDelta::Pixels { x, y } => (x, y),
        }
    }

    /// The gesture a listener hears for it.
    pub fn gesture(&self, at: Point, held: Modifiers, detent_px: f64) -> Gesture {
        let (x, y) = self.motion(detent_px);
        Gesture::Scroll {
            phase: self.phase,
            by: Point {
                x: Px(x as f32),
                y: Px(y as f32),
            },
            at,
            held,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NONE: Modifiers = Modifiers::empty();

    fn input(delta: WheelDelta, phase: GesturePhase) -> WheelInput {
        WheelInput { delta, phase }
    }

    fn detent(axis: ScrollAxis, steps: f64) -> Vec<PointerScroll> {
        vec![PointerScroll::Detents { axis, steps }]
    }

    fn pan(dx: f64, dy: f64) -> PointerScroll {
        PointerScroll::Pan {
            dx,
            dy,
            kinetic: Kinetic::Momentum,
        }
    }

    #[test]
    fn what_the_engine_does_for_a_wheel_or_a_touchpad() {
        // name, event, modifiers held, what the engine is asked
        let cases = [
            (
                "a click down is a detent toward the maximum",
                input(WheelDelta::Lines { x: 0.0, y: -1.0 }, GesturePhase::Changed),
                NONE,
                detent(ScrollAxis::Y, 1.0),
            ),
            (
                "a click up",
                input(WheelDelta::Lines { x: 0.0, y: 2.0 }, GesturePhase::Changed),
                NONE,
                detent(ScrollAxis::Y, -2.0),
            ),
            (
                "a tilt is horizontal",
                input(WheelDelta::Lines { x: -1.0, y: 0.0 }, GesturePhase::Changed),
                NONE,
                detent(ScrollAxis::X, 1.0),
            ),
            (
                "shift turns a vertical wheel horizontal",
                input(WheelDelta::Lines { x: 0.0, y: -1.0 }, GesturePhase::Changed),
                Modifiers::SHIFT,
                detent(ScrollAxis::X, 1.0),
            ),
            (
                "a wheel that did not turn",
                input(WheelDelta::Lines { x: 0.0, y: 0.0 }, GesturePhase::Changed),
                NONE,
                Vec::new(),
            ),
            (
                "the fingers' first motion",
                input(WheelDelta::Pixels { x: 3.0, y: -8.0 }, GesturePhase::Began),
                NONE,
                vec![pan(-3.0, 8.0)],
            ),
            (
                "the fingers lift with nothing more to say",
                input(WheelDelta::Pixels { x: 0.0, y: 0.0 }, GesturePhase::Ended),
                NONE,
                vec![PointerScroll::Stop],
            ),
            (
                "the fingers lift with a last motion",
                input(WheelDelta::Pixels { x: 0.0, y: -2.0 }, GesturePhase::Ended),
                NONE,
                vec![pan(0.0, 2.0), PointerScroll::Stop],
            ),
            (
                "a cancelled gesture ends too",
                input(
                    WheelDelta::Pixels { x: 0.0, y: 0.0 },
                    GesturePhase::Cancelled,
                ),
                NONE,
                vec![PointerScroll::Stop],
            ),
        ];
        for (name, event, held, want) in cases {
            assert_eq!(pointer_scrolls(event, held), want, "{name}");
        }
    }

    #[test]
    fn a_listener_hears_a_click_as_one_detent_of_content_motion() {
        const AT: Point = Point {
            x: Px(10.0),
            y: Px(20.0),
        };
        // name, event, the motion `by` carries
        let cases = [
            (
                "a click is a detent",
                input(WheelDelta::Lines { x: 0.0, y: 3.0 }, GesturePhase::Changed),
                (0.0, 180.0),
            ),
            (
                "pixels are their own",
                input(
                    WheelDelta::Pixels { x: 20.0, y: -40.0 },
                    GesturePhase::Ended,
                ),
                (20.0, -40.0),
            ),
        ];
        for (name, event, (x, y)) in cases {
            let got = event.gesture(AT, Modifiers::CONTROL, 60.0);
            let want = Gesture::Scroll {
                phase: event.phase,
                by: Point { x: Px(x), y: Px(y) },
                at: AT,
                held: Modifiers::CONTROL,
            };
            assert_eq!(got, want, "{name}");
        }
    }
}
