//! A dragged thing going home (design/27 section 3.12 rule 3; design/10 for the dock): while a
//! hand drags it, it follows 1:1 as an offset from its place; let go where it does not drop, it
//! springs back to its place on a two-dimensional spring, each axis starting at the velocity the
//! hand had along it. Picked up again on the way, it follows from where it is.
//!
//! The dock's tile return is the first user (sill wires the drag; quire draws the offset with
//! [`crate::components::overlays::drag_ghost::DragReturnFrame`]).

use super::spring::SpringPhase;
use super::spring_point::{Release, SpringPointMotion, use_spring_point_motion};
use super::spring_spec::SpringResponse;
use crate::core::geometry::units::{Point, Px};
use crate::motion::detail::touch::Touch;

/// Its place: no offset.
const HOME: Point = Point {
    x: Px(0.0),
    y: Px(0.0),
};

/// A drag's way home: an offset from the thing's place, driven by a two-dimensional spring.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragReturn {
    spring: SpringPointMotion,
}

impl DragReturn {
    /// Follow the hand: the thing is drawn `offset` from its place, with no spring.
    pub fn follow(self, offset: Point) {
        self.spring.track(offset);
    }

    /// It was let go where nothing took it: spring home. A hand's release (`Touch::Contact`
    /// from `onpointerup`) hands each axis the velocity along it; anything else springs home
    /// critically from where it is.
    pub fn home(self, touch: Touch, release: Release) {
        self.spring.go(HOME, touch, release, SpringResponse::Move);
    }

    /// Stand at its place at once (it dropped somewhere, and the host moves it there).
    pub fn settle(self) {
        self.spring.snap(HOME);
    }

    /// Where it is drawn from its place now, subscribing the caller's render to the springs.
    pub fn offset(self) -> Point {
        self.spring.frame().at
    }

    /// Moving while either axis moves.
    pub fn phase(self) -> SpringPhase {
        self.spring.peek().phase
    }
}

/// A drag's way home, standing at its place.
pub fn use_drag_return() -> DragReturn {
    DragReturn {
        spring: use_spring_point_motion(HOME),
    }
}
