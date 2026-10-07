//! The harness's scroll wheel: a touchpad or a mouse wheel delta delivered where the pointer
//! is, as the window delivers winit's `MouseWheel` (a horizontal scroll swipes a
//! notification away), and a wheel's clicks, which the scroll engine takes as it does in a
//! window.

use crate::harness::Harness;
use crate::input::PointerAction;
use crate::input::PointerInput;
use blitz_traits::events::{BlitzWheelDelta, BlitzWheelEvent, PointerCoords, UiEvent};
use ds::host::gesture::{Gesture, GesturePhase, ScrollSource};
use ds::prelude::*;
use ds_blitz::seam::{WheelDelta, WheelInput, WheelUse};
use ds_core::time::clock::now;
use keyboard_types::Modifiers;

impl Harness {
    /// Scroll by `dx`, `dy` pixels with the pointer at `at`. Like the window, Blitz hands the
    /// delta to the element under the pointer (the hover node, which only a pointer move sets,
    /// so the pointer is moved to `at` first, as a window has always delivered a move there) and
    /// carries no scroll phase: a gesture's end is only the deltas stopping.
    pub(crate) fn wheel(&mut self, at: Point, dx: Px, dy: Px) {
        self.pointer(PointerInput {
            at,
            action: PointerAction::Move,
            mods: Modifiers::empty(),
        });
        let (x, y) = (at.x.0, at.y.0);
        self.deliver(UiEvent::Wheel(BlitzWheelEvent {
            delta: BlitzWheelDelta::Pixels(f64::from(dx.0), f64::from(dy.0)),
            coords: PointerCoords {
                page_x: x,
                page_y: y,
                screen_x: x,
                screen_y: y,
                client_x: x,
                client_y: y,
            },
            buttons: self.held_buttons().blitz(),
            mods: Modifiers::empty(),
            element: Default::default(),
        }));
        self.gesture(Gesture::Scroll {
            source: ScrollSource::Finger,
            phase: GesturePhase::Changed,
            by: Point { x: dx, y: dy },
            at,
            held: Modifiers::empty(),
        });
    }

    /// Turn a wheel `x`, `y` clicks (winit's sign) with the pointer at `at`. The pointer is moved
    /// there first. The engine scrolls for it and the document never sees the wheel, as in a
    /// window; over a `data-wheel="capture"` element the raw wheel goes to the document instead.
    pub(crate) fn detents(&mut self, at: Point, x: f64, y: f64, held: Modifiers) {
        self.pointer(PointerInput {
            at,
            action: PointerAction::Move,
            mods: Modifiers::empty(),
        });
        self.doc.scroll.track_modifiers(held);
        let input = WheelInput {
            delta: WheelDelta::Lines { x, y },
            phase: GesturePhase::Changed,
        };
        let (used, _) = self.doc.scroll.wheel(input, now());
        if used == WheelUse::Passed {
            let (px, py) = (at.x.0, at.y.0);
            self.deliver(UiEvent::Wheel(BlitzWheelEvent {
                delta: BlitzWheelDelta::Lines(x, y),
                coords: PointerCoords {
                    page_x: px,
                    page_y: py,
                    screen_x: px,
                    screen_y: py,
                    client_x: px,
                    client_y: py,
                },
                buttons: self.held_buttons().blitz(),
                mods: Modifiers::empty(),
                element: Default::default(),
            }));
        }
        self.settle_now();
    }

    /// Fingers on a touchpad move `dx`, `dy` in `phase`, as the window delivers winit's
    /// `MouseWheel` with pixels: the engine takes it, and the pointer is moved to `at` first.
    pub(crate) fn fingers(&mut self, at: Point, dx: Px, dy: Px, phase: GesturePhase) {
        self.pointer(PointerInput {
            at,
            action: PointerAction::Move,
            mods: Modifiers::empty(),
        });
        let input = WheelInput {
            delta: WheelDelta::Pixels {
                x: f64::from(dx.0),
                y: f64::from(dy.0),
            },
            phase,
        };
        let _ = self.doc.scroll.wheel(input, now());
        self.settle_now();
    }

    /// Publish `gesture` to the components listening, as the window does for winit's pinch and
    /// wheel events, and bring the document up to date.
    pub(crate) fn gesture(&mut self, gesture: Gesture) {
        let bus = self.doc.gestures.clone();
        self.doc.doc.vdom.in_runtime(|| bus.publish(gesture));
        self.settle_now();
    }
}
