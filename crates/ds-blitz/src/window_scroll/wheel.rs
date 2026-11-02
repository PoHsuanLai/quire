//! A wheel or touchpad event as the scroll engine and the gesture listeners take it: winit's
//! `MouseWheel` reduced to what it means. Pure.

use blitz_kit::scroll::engine::Kinetic;
use blitz_kit::scroll::geom::ScrollAxis;
use blitz_kit::scroll::pad::PointerScroll;
use blitz_kit::scroll::time::Elapsed;
use ds::host::gesture::{Gesture, GesturePhase, ScrollSource};
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

/// What a touchpad event is in a run on a system that follows the fingers' lift with momentum
/// of its own (macOS): winit reports that momentum as ordinary pixel deltas with
/// `Started`, `Moved` and `Ended` (`winit-appkit` maps `momentumPhase` first, then `phase`),
/// with nothing to tell it from the fingers' own events but where it falls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// The fingers' own motion, or any event on a system that sends no momentum.
    Finger,
    /// The fingers lifted: the system's momentum may follow.
    Lift,
    /// The system's momentum.
    Tail,
    /// The system's momentum ended.
    TailEnd,
}

/// Where a touchpad run is, to tell the system's momentum from the fingers.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Run {
    #[default]
    Idle,
    /// The fingers are down; `last` is their latest nonzero motion.
    Down { last: (f64, f64) },
    /// The fingers lifted at `at` moving as `last`.
    Lifted { at: Elapsed, last: (f64, f64) },
    /// The system's momentum runs.
    Tail,
}

impl Run {
    /// This run after `input` at `now`, and what the event is. Momentum is the events that
    /// follow a lift within `grace` seconds and move the way the fingers did (a new touch is a
    /// `Started` with no motion, or against the last, or later); it runs to its own `Ended`.
    pub fn hear(self, input: WheelInput, now: Elapsed, grace: f64) -> (Run, Role) {
        let WheelDelta::Pixels { x, y } = input.delta else {
            return (self, Role::Finger);
        };
        let moved = (x, y) != (0.0, 0.0);
        let with = |last: (f64, f64)| match moved {
            true => (x, y),
            false => last,
        };
        let following = |at: Elapsed, last: (f64, f64)| {
            let gap = now.0.as_secs_f64() - at.0.as_secs_f64();
            moved && gap <= grace && x * last.0 + y * last.1 > 0.0
        };
        let run = match self {
            Run::Lifted { at, last }
                if matches!(input.phase, GesturePhase::Began | GesturePhase::Changed)
                    && following(at, last) =>
            {
                Run::Tail
            }
            Run::Lifted { .. } => Run::Idle,
            other => other,
        };
        match (run, input.phase) {
            (Run::Tail, GesturePhase::Ended | GesturePhase::Cancelled) => {
                (Run::Idle, Role::TailEnd)
            }
            (Run::Tail, GesturePhase::Changed) => (Run::Tail, Role::Tail),
            // A touch while the momentum runs, or the first of it just recognised.
            (Run::Tail, GesturePhase::Began) if matches!(self, Run::Lifted { .. }) => {
                (Run::Tail, Role::Tail)
            }
            (_, GesturePhase::Began) => (
                Run::Down {
                    last: with((0.0, 0.0)),
                },
                Role::Finger,
            ),
            (Run::Down { last }, GesturePhase::Changed) => {
                (Run::Down { last: with(last) }, Role::Finger)
            }
            (Run::Down { last }, GesturePhase::Ended) => (
                Run::Lifted {
                    at: now,
                    last: with(last),
                },
                Role::Lift,
            ),
            (_, GesturePhase::Cancelled | GesturePhase::Ended) => (Run::Idle, Role::Finger),
            (other, GesturePhase::Changed) => (other, Role::Finger),
        }
    }
}

/// What the engine does for `input` with `held` modifiers: wheel clicks become detents (Shift
/// turns a vertical wheel horizontal), a touchpad's pixels become a phased pan that ends with the
/// fingers' lift, or the system's momentum after it (`role`).
pub fn pointer_scrolls(input: WheelInput, held: Modifiers, role: Role) -> Vec<PointerScroll> {
    match input.delta {
        WheelDelta::Lines { x, y } => detents(-x, -y, held),
        WheelDelta::Pixels { x, y } => pan(-x, -y, input.phase, role),
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
fn pan(dx: f64, dy: f64, phase: GesturePhase, role: Role) -> Vec<PointerScroll> {
    let here = dx != 0.0 || dy != 0.0;
    let moved = here.then_some(match role {
        Role::Tail | Role::TailEnd => PointerScroll::Tail { dx, dy },
        Role::Finger | Role::Lift => PointerScroll::Pan {
            dx,
            dy,
            kinetic: Kinetic::Momentum,
        },
    });
    let lifted = match (role, phase) {
        (Role::Lift, _) => Some(PointerScroll::Lift),
        (Role::TailEnd, _) => Some(PointerScroll::Stop),
        (_, GesturePhase::Ended | GesturePhase::Cancelled) => Some(PointerScroll::Stop),
        (_, GesturePhase::Began | GesturePhase::Changed) => None,
    };
    moved.into_iter().chain(lifted).collect()
}

impl WheelInput {
    /// Which device this event is from: clicks are a wheel's, pixels are fingers' (winit reports
    /// no other difference).
    pub fn source(&self) -> ScrollSource {
        match self.delta {
            WheelDelta::Lines { .. } => ScrollSource::Wheel,
            WheelDelta::Pixels { .. } => ScrollSource::Finger,
        }
    }

    /// How many clicks the wheel turned, whichever way (0 for a touchpad).
    pub fn turned(&self) -> f64 {
        match self.delta {
            WheelDelta::Lines { x, y } => x.abs().max(y.abs()),
            WheelDelta::Pixels { .. } => 0.0,
        }
    }

    /// The same turn carried `gain` times as far; a touchpad's pixels are its own.
    pub fn scaled(self, gain: f64) -> WheelInput {
        let delta = match self.delta {
            WheelDelta::Lines { x, y } => WheelDelta::Lines {
                x: x * gain,
                y: y * gain,
            },
            WheelDelta::Pixels { .. } => self.delta,
        };
        WheelInput { delta, ..self }
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
            source: self.source(),
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
            assert_eq!(pointer_scrolls(event, held, Role::Finger), want, "{name}");
        }
    }

    #[test]
    fn a_turn_scales_by_its_gain_and_a_touchpad_slide_does_not() {
        let wheel = input(WheelDelta::Lines { x: 0.0, y: -2.0 }, GesturePhase::Changed);
        assert_eq!(wheel.turned(), 2.0);
        assert_eq!(
            wheel.scaled(2.5).delta,
            WheelDelta::Lines { x: 0.0, y: -5.0 }
        );
        let pad = input(
            WheelDelta::Pixels { x: 1.0, y: -4.0 },
            GesturePhase::Changed,
        );
        assert_eq!(pad.turned(), 0.0);
        assert_eq!(pad.scaled(2.0), pad);
    }

    #[test]
    fn a_listener_hears_a_click_as_one_detent_of_content_motion() {
        const AT: Point = Point {
            x: Px(10.0),
            y: Px(20.0),
        };
        // name, event, the motion `by` carries, the source a listener is told
        let cases = [
            (
                "a click is a detent of a wheel",
                input(WheelDelta::Lines { x: 0.0, y: 3.0 }, GesturePhase::Changed),
                (0.0, 180.0),
                ScrollSource::Wheel,
            ),
            (
                "pixels are their own, and the fingers'",
                input(
                    WheelDelta::Pixels { x: 20.0, y: -40.0 },
                    GesturePhase::Ended,
                ),
                (20.0, -40.0),
                ScrollSource::Finger,
            ),
        ];
        for (name, event, (x, y), source) in cases {
            let got = event.gesture(AT, Modifiers::CONTROL, 60.0);
            let want = Gesture::Scroll {
                source,
                phase: event.phase,
                by: Point { x: Px(x), y: Px(y) },
                at: AT,
                held: Modifiers::CONTROL,
            };
            assert_eq!(got, want, "{name}");
        }
    }

    fn at(ms: u64) -> Elapsed {
        Elapsed(std::time::Duration::from_millis(ms))
    }

    fn pixels(y: f64, phase: GesturePhase) -> WheelInput {
        input(WheelDelta::Pixels { x: 0.0, y }, phase)
    }

    /// Feed `events` (ms, dy, phase) through a run with the default 150 ms grace.
    fn roles(events: &[(u64, f64, GesturePhase)]) -> Vec<Role> {
        let mut run = Run::default();
        events
            .iter()
            .map(|&(ms, y, phase)| {
                let (next, role) = run.hear(pixels(y, phase), at(ms), 0.15);
                run = next;
                role
            })
            .collect()
    }

    #[test]
    fn the_systems_momentum_after_a_lift_is_told_from_the_fingers() {
        use GesturePhase::{Began, Changed, Ended};
        let flick = [
            (0, 5.0, Began),
            (8, 20.0, Changed),
            (16, 30.0, Changed),
            (24, 0.0, Ended),
            // momentum: Started, Moved.., Ended
            (40, 28.0, Began),
            (48, 26.0, Changed),
            (56, 24.0, Changed),
            (64, 0.0, Ended),
        ];
        assert_eq!(
            roles(&flick),
            [
                Role::Finger,
                Role::Finger,
                Role::Finger,
                Role::Lift,
                Role::Tail,
                Role::Tail,
                Role::Tail,
                Role::TailEnd
            ]
        );
    }

    #[test]
    fn a_new_touch_is_not_momentum() {
        use GesturePhase::{Began, Changed, Ended};
        let lift = [(0, 20.0, Began), (8, 30.0, Changed), (16, 0.0, Ended)];
        let after = |next: (u64, f64, GesturePhase)| {
            let mut events = lift.to_vec();
            events.push(next);
            *roles(&events).last().expect("roles")
        };
        // name, the event after the lift, its role
        let cases = [
            ("a touch with no motion", (30, 0.0, Began), Role::Finger),
            ("the other way", (30, -20.0, Began), Role::Finger),
            ("too late", (400, 20.0, Began), Role::Finger),
            ("momentum straight after", (30, 20.0, Began), Role::Tail),
            (
                "momentum whose start was merged",
                (30, 20.0, Changed),
                Role::Tail,
            ),
        ];
        for (name, event, want) in cases {
            assert_eq!(after(event), want, "{name}");
        }
    }

    #[test]
    fn a_touch_during_the_momentum_starts_a_new_run() {
        use GesturePhase::{Began, Changed, Ended};
        let events = [
            (0, 30.0, Began),
            (8, 30.0, Changed),
            (16, 0.0, Ended),
            (30, 25.0, Began),
            (38, 20.0, Changed),
            (60, 3.0, Began),
            (68, 4.0, Changed),
            (76, 0.0, Ended),
        ];
        let got = roles(&events);
        assert_eq!(
            &got[3..],
            [
                Role::Tail,
                Role::Tail,
                Role::Finger,
                Role::Finger,
                Role::Lift
            ]
        );
    }

    #[test]
    fn a_wheel_or_a_run_with_no_lift_is_the_fingers() {
        let mut run = Run::default();
        let wheel = input(WheelDelta::Lines { x: 0.0, y: -1.0 }, GesturePhase::Changed);
        assert_eq!(run.hear(wheel, at(0), 0.15), (Run::Idle, Role::Finger));
        run = Run::default();
        // A mouse that reports pixels with no phase: Changed forever, never Ended.
        let (run, role) = run.hear(pixels(5.0, GesturePhase::Changed), at(0), 0.15);
        assert_eq!((run, role), (Run::Idle, Role::Finger));
    }

    #[test]
    fn the_roles_make_the_engines_inputs() {
        let ended = input(WheelDelta::Pixels { x: 0.0, y: -2.0 }, GesturePhase::Ended);
        let tail = PointerScroll::Tail { dx: 0.0, dy: 2.0 };
        // name, role, what the engine is asked
        let cases = [
            (
                "the lift",
                Role::Lift,
                vec![pan(0.0, 2.0), PointerScroll::Lift],
            ),
            (
                "the momentum's end",
                Role::TailEnd,
                vec![tail, PointerScroll::Stop],
            ),
        ];
        for (name, role, want) in cases {
            assert_eq!(pointer_scrolls(ended, NONE, role), want, "{name}");
        }
        let moving = input(
            WheelDelta::Pixels { x: 0.0, y: -2.0 },
            GesturePhase::Changed,
        );
        assert_eq!(pointer_scrolls(moving, NONE, Role::Tail), vec![tail]);
    }
}
