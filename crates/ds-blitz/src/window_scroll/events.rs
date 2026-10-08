//! winit's events as window scrolling takes them.

use std::time::Instant;

use blitz_kit::scroll::driver::KeyRepeat;
use blitz_kit::scroll::geom::ViewPoint;
use dioxus_native::winit::event::{ElementState, MouseScrollDelta, TouchPhase, WindowEvent};

use super::keys::scroll_key_of;
use super::state::{Frames, WheelUse, WindowScroll};
use super::wheel::{WheelDelta, WheelInput};
use crate::edit_window::modifiers_of;
use crate::gesture_window::phase_of;

/// Whether the document still gets an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intercept {
    /// Scrolling took it: the document must not see it.
    Taken,
    /// The document hears it as usual.
    Passed,
}

/// What window scrolling made of one event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Handled {
    pub event: Intercept,
    /// Whether the window must draw another frame for scrolling.
    pub frames: Frames,
}

impl Handled {
    const PASSED: Handled = Handled {
        event: Intercept::Passed,
        frames: Frames::Idle,
    };
}

impl WindowScroll {
    /// Scrolling's part in `event` at display scale `scale`: it follows the pointer and the
    /// modifiers, takes the wheel, acts on scroll keys (which the document hears as well). The
    /// step before each frame is not here: the window runs [`WindowScroll::frame`] itself, ahead
    /// of the frame's renders.
    pub fn event(&self, event: &WindowEvent, scale: f64, now: Instant) -> Handled {
        let scale = scale.max(f64::EPSILON);
        match event {
            WindowEvent::PointerMoved { position, .. } => {
                self.track_pointer(ViewPoint {
                    x: position.x / scale,
                    y: position.y / scale,
                });
                Handled::PASSED
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.track_modifiers(modifiers_of(modifiers.state()));
                Handled::PASSED
            }
            WindowEvent::MouseWheel { delta, phase, .. } => {
                let Some(input) = wheel_of(delta, *phase, scale) else {
                    return Handled::PASSED;
                };
                let (used, frames) = self.wheel(input, now);
                let event = match used {
                    WheelUse::Taken => Intercept::Taken,
                    WheelUse::Passed => Intercept::Passed,
                };
                Handled { event, frames }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let (_, held) = self.pointer();
                let frames = match scroll_key_of(&event.logical_key, held) {
                    Some(key) if event.state == ElementState::Pressed => {
                        self.key(key, repeat_of(event.repeat), now)
                    }
                    Some(_) => self.key_up(now),
                    None => Frames::Idle,
                };
                Handled {
                    event: Intercept::Passed,
                    frames,
                }
            }
            _ => Handled::PASSED,
        }
    }
}

/// A key press the system repeats, or the first.
fn repeat_of(repeat: bool) -> KeyRepeat {
    match repeat {
        true => KeyRepeat::Repeated,
        false => KeyRepeat::First,
    }
}

/// The wheel input winit's `delta` and `phase` make, with pixels in logical px at `scale`;
/// `None` for a delta kind this winit does not name.
fn wheel_of(delta: &MouseScrollDelta, phase: TouchPhase, scale: f64) -> Option<WheelInput> {
    let delta = match delta {
        MouseScrollDelta::LineDelta(x, y) => WheelDelta::Lines {
            x: f64::from(*x),
            y: f64::from(*y),
        },
        MouseScrollDelta::PixelDelta(pixels) => WheelDelta::Pixels {
            x: pixels.x / scale,
            y: pixels.y / scale,
        },
        _ => return None,
    };
    let phase = phase_of(phase);
    Some(WheelInput { delta, phase })
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus_native::winit::dpi::PhysicalPosition;
    use ds::host::gesture::GesturePhase;

    #[test]
    fn winit_wheels_become_wheel_inputs() {
        // name, delta, phase, scale, expected
        let cases = [
            (
                "clicks are clicks",
                MouseScrollDelta::LineDelta(0.0, 3.0),
                TouchPhase::Moved,
                2.0,
                WheelInput {
                    delta: WheelDelta::Lines { x: 0.0, y: 3.0 },
                    phase: GesturePhase::Changed,
                },
            ),
            (
                "pixels are logical at 2x",
                MouseScrollDelta::PixelDelta(PhysicalPosition::new(40.0, -80.0)),
                TouchPhase::Ended,
                2.0,
                WheelInput {
                    delta: WheelDelta::Pixels { x: 20.0, y: -40.0 },
                    phase: GesturePhase::Ended,
                },
            ),
        ];
        for (name, delta, phase, scale, want) in cases {
            assert_eq!(wheel_of(&delta, phase, scale), Some(want), "{name}");
        }
    }

    #[test]
    fn a_key_repeat_is_a_repeat() {
        assert_eq!(repeat_of(true), KeyRepeat::Repeated);
        assert_eq!(repeat_of(false), KeyRepeat::First);
    }
}
