//! A spring in two dimensions (design/05 section 14.3): two springs, one per axis,
//! moved together. A dragged widget springs into its snap cell and the others glide aside;
//! retargeting mid-glide keeps each axis's position and velocity, a hand's release hands each
//! axis the velocity along it, Reduced is critically damped with no thrown velocity, and nothing
//! asks for frames at rest.

use super::projection::Throw;
use super::spring::SpringPhase;
use super::spring_spec::{SpringResponse, SpringSpec};
use super::use_spring::{PxPerUnit, SpringMotion, use_spring_motion};
use super::velocity::Velocity;
use crate::detail::touch::Touch;
use crate::geometry::units::{Point, Px};
use dioxus::core::queue_effect;
use dioxus::prelude::*;

/// How the hand was moving when it let go, per axis, in pixels per second.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Release {
    /// Along x, right positive.
    pub x: Velocity,
    /// Along y, down positive.
    pub y: Velocity,
}

/// A throw in two dimensions: where the hand let go and how fast it was going.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointThrow {
    /// Where the thing was when the hand let go.
    pub from: Point,
    /// How fast the hand was moving along each axis.
    pub release: Release,
}

impl PointThrow {
    /// Where the throw would come to rest, each axis projected at `r = 0.998` per ms.
    pub fn projected(self) -> Point {
        let along = |from, velocity| Throw { from, velocity }.projected();
        Point {
            x: along(self.from.x, self.release.x),
            y: along(self.from.y, self.release.y),
        }
    }

    /// The point of `ends` nearest the projection (a snap cell); `None` when there are none.
    pub fn landing(self, ends: &[Point]) -> Option<Point> {
        let p = self.projected();
        let distance = |q: &Point| (q.x.0 - p.x.0).powi(2) + (q.y.0 - p.y.0).powi(2);
        ends.iter()
            .copied()
            .min_by(|a, b| distance(a).total_cmp(&distance(b)))
    }
}

/// Where a two-dimensional spring is drawn now.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointFrame {
    /// The position.
    pub at: Point,
    /// Moving while either axis moves.
    pub phase: SpringPhase,
}

/// A two-dimensional spring a component drives.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringPointMotion {
    x: SpringMotion,
    y: SpringMotion,
}

impl SpringPointMotion {
    /// Spring to `target` on `response`: from where it is at the speed it has on each axis, or,
    /// for a contact, at the hand's `release` velocity along each axis (a remote touch ignores
    /// `release`).
    pub fn go(self, target: Point, touch: Touch, release: Release, response: SpringResponse) {
        let along = |velocity| {
            let touch = match touch {
                Touch::Contact(contact) => Touch::Contact(contact.with_velocity(velocity)),
                Touch::Remote => Touch::Remote,
            };
            SpringSpec::for_touch(touch).response(response)
        };
        self.x.go(target.x.0, along(release.x));
        self.y.go(target.y.0, along(release.y));
    }

    /// Follow a hand 1:1: stand at `at`, no frames of its own.
    pub fn track(self, at: Point) {
        self.x.track(at.x.0);
        self.y.track(at.y.0);
    }

    /// Stand at `at` at once.
    pub fn snap(self, at: Point) {
        self.x.snap(at.x.0);
        self.y.snap(at.y.0);
    }

    /// Where it is, subscribing the caller's render to its frames.
    pub fn frame(self) -> PointFrame {
        let (x, y) = (self.x.frame(), self.y.frame());
        PointFrame {
            at: Point {
                x: Px(x.position()),
                y: Px(y.position()),
            },
            phase: both(x.phase(), y.phase()),
        }
    }

    /// Where it is, without subscribing.
    pub fn peek(self) -> PointFrame {
        let (x, y) = (self.x.peek(), self.y.peek());
        PointFrame {
            at: Point {
                x: Px(x.position()),
                y: Px(y.position()),
            },
            phase: both(x.phase(), y.phase()),
        }
    }
}

fn both(x: SpringPhase, y: SpringPhase) -> SpringPhase {
    match (x, y) {
        (SpringPhase::Rest, SpringPhase::Rest) => SpringPhase::Rest,
        _ => SpringPhase::Moving,
    }
}

/// A two-dimensional spring standing at `at`, in pixels.
pub fn use_spring_point_motion(at: Point) -> SpringPointMotion {
    SpringPointMotion {
        x: use_spring_motion(at.x.0, PxPerUnit(1.0)),
        y: use_spring_motion(at.y.0, PxPerUnit(1.0)),
    }
}

/// A two-dimensional spring that follows `target` (a widget's cell): it stands there on mount
/// and glides to each new target from where it is, at the speed it has, on the spring `spec`
/// chooses. For a throw into a cell, use [`use_spring_point_motion`] and `go` with the release.
pub fn use_spring_point(target: Point, spec: SpringSpec) -> PointFrame {
    let motion = use_spring_point_motion(target);
    let mut seen = use_hook(|| CopyValue::new(target));
    if *seen.peek() != target {
        seen.set(target);
        queue_effect(move || {
            motion.x.go(target.x.0, spec);
            motion.y.go(target.y.0, spec);
        });
    }
    motion.frame()
}

#[cfg(test)]
mod tests {
    use super::{PointThrow, Release};
    use crate::geometry::units::{Point, Px};
    use crate::motion::velocity::Velocity;

    fn pt(x: f32, y: f32) -> Point {
        Point { x: Px(x), y: Px(y) }
    }

    /// A release (where, how fast) and the cell it should land in.
    type Case = ((f32, f32), (i32, i32), (f32, f32));

    #[test]
    fn a_throw_lands_in_the_cell_nearest_its_projection() {
        let cells = [
            pt(0.0, 0.0),
            pt(200.0, 0.0),
            pt(0.0, 200.0),
            pt(200.0, 200.0),
        ];
        const CASES: &[Case] = &[
            ((60.0, 60.0), (0, 0), (0.0, 0.0)),
            ((60.0, 60.0), (600, 0), (200.0, 0.0)),
            ((60.0, 60.0), (600, 600), (200.0, 200.0)),
            ((150.0, 150.0), (-800, 0), (0.0, 200.0)),
        ];
        for &((x, y), (vx, vy), (wx, wy)) in CASES {
            let throw = PointThrow {
                from: pt(x, y),
                release: Release {
                    x: Velocity(vx),
                    y: Velocity(vy),
                },
            };
            assert_eq!(
                throw.landing(&cells),
                Some(pt(wx, wy)),
                "{x},{y} at {vx},{vy}"
            );
        }
    }
}
