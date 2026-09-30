//! The harness's scroll wheel: a touchpad or a mouse wheel delta delivered where the pointer
//! is, as the window delivers winit's `MouseWheel` (a horizontal scroll swipes a
//! notification away).

use crate::harness::Harness;
use crate::input::PointerAction;
use crate::input::PointerInput;
use blitz_traits::events::{BlitzWheelDelta, BlitzWheelEvent, PointerCoords, UiEvent};
use ds::prelude::*;
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
    }
}
