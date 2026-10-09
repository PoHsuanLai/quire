//! The fingers' run as an eased listener hears it (`WheelDelivery::Eased`): the fingers' own
//! motion while they are down, then the engine's glide after they lift, ending with one `Ended`
//! when the glide is over. The speed and the curve are the engine's (`blitz_kit::scroll`), so
//! content a listener moves coasts as far as a native container does. There is no stretch past
//! an edge: the listener's bounds are its own. Pure.

use blitz_kit::scroll::config::ScrollMomentum;
use blitz_kit::scroll::engine::{Motion, Physics};
use blitz_kit::scroll::time::Elapsed;
use blitz_kit::scroll::velocity::{SampleTime, Samples};
use ds::host::gesture::GesturePhase;

/// A distance too small to say anything about: the closed form of the glide leaves this much
/// of rounding at its start.
const DUST_PX: f64 = 1e-6;

/// What an eased listener hears: where its run is, and how far the content moves, in winit's
/// sign.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Heard {
    pub phase: GesturePhase,
    pub by: (f64, f64),
}

/// One axis of the fingers: how far they have moved in all, and when.
#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct Track {
    x: Axis,
    y: Axis,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct Axis {
    total: f64,
    samples: Samples,
}

impl Track {
    /// This track after the fingers moved by `(dx, dy)` at `now`.
    fn moved(mut self, (dx, dy): (f64, f64), now: Elapsed) -> Track {
        let at = SampleTime(now.0);
        self.x.total += dx;
        self.y.total += dy;
        self.x.samples.push(at, self.x.total);
        self.y.samples.push(at, self.y.total);
        self
    }

    /// The speed each axis had as the fingers lifted at `now`, px/s signed, zero for one too
    /// slow to glide or when the engine's momentum is off.
    fn release(&self, now: Elapsed, physics: &Physics) -> (f64, f64) {
        let at = SampleTime(now.0);
        let speed = |axis: &Axis| {
            let v = physics.fling(axis.samples.velocity(at, physics.release));
            match (physics.momentum, v.abs() >= physics.start) {
                (ScrollMomentum::On, true) => v,
                _ => 0.0,
            }
        };
        (speed(&self.x), speed(&self.y))
    }
}

/// A glide from `v0` that began at `since`, and how far of it has been heard.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Gliding {
    v0: (f64, f64),
    since: Elapsed,
    heard: (f64, f64),
}

/// Where the fingers' run is.
#[derive(Debug, Clone, Default, PartialEq)]
pub(super) enum Coast {
    #[default]
    Idle,
    /// The fingers are down.
    Down(Track),
    /// They lifted, and the content still glides.
    Gliding(Gliding),
}

impl Coast {
    /// The fingers did `phase` and moved by `by` at `now`: the new state and what an eased
    /// listener hears. A touch during a glide ends it first.
    pub(super) fn touch(
        self,
        phase: GesturePhase,
        by: (f64, f64),
        now: Elapsed,
        physics: &Physics,
    ) -> (Coast, Vec<Heard>) {
        let (state, mut heard) = match self {
            Coast::Gliding(_) => (Coast::Idle, vec![heard(GesturePhase::Ended, (0.0, 0.0))]),
            other => (other, Vec::new()),
        };
        let track = match state {
            Coast::Down(track) if phase != GesturePhase::Began => track,
            _ => Track::default(),
        }
        .moved(by, now);
        let next = match phase {
            GesturePhase::Began | GesturePhase::Changed => {
                heard.push(self::heard(phase, by));
                Coast::Down(track)
            }
            GesturePhase::Cancelled => {
                heard.push(self::heard(phase, by));
                Coast::Idle
            }
            GesturePhase::Ended => {
                let (next, said) = lift(&track, by, now, physics);
                heard.extend(said);
                next
            }
        };
        (next, heard)
    }

    /// A frame at `now`: what the glide moved since the last, and `Ended` on the frame it ran
    /// out.
    pub(super) fn advance(self, now: Elapsed, physics: &Physics) -> (Coast, Vec<Heard>) {
        let Coast::Gliding(glide) = self else {
            return (self, Vec::new());
        };
        let secs = now.0.saturating_sub(glide.since.0).as_secs_f64();
        let at = |v0: f64| v0.signum() * physics.glide.travel(v0.abs(), secs);
        let (x, y) = (at(glide.v0.0), at(glide.v0.1));
        let by = (x - glide.heard.0, y - glide.heard.1);
        let left = |v0: f64| physics.glide.duration(v0.abs()) > secs;
        let worth = by.0.abs() >= DUST_PX || by.1.abs() >= DUST_PX;
        let moved = worth.then(|| heard(GesturePhase::Changed, by));
        let heard_now = if worth { (x, y) } else { glide.heard };
        match left(glide.v0.0) || left(glide.v0.1) {
            true => (
                Coast::Gliding(Gliding {
                    heard: heard_now,
                    ..glide
                }),
                moved.into_iter().collect(),
            ),
            false => (
                Coast::Idle,
                moved
                    .into_iter()
                    .chain([heard(GesturePhase::Ended, (0.0, 0.0))])
                    .collect(),
            ),
        }
    }

    /// Whether a frame is wanted.
    pub(super) fn motion(&self) -> Motion {
        match self {
            Coast::Gliding(_) => Motion::Animating,
            Coast::Idle | Coast::Down(_) => Motion::Still,
        }
    }
}

fn heard(phase: GesturePhase, by: (f64, f64)) -> Heard {
    Heard { phase, by }
}

/// The fingers lifted with `by` as their last motion: a glide if they were fast enough, else the
/// end of the run.
fn lift(track: &Track, by: (f64, f64), now: Elapsed, physics: &Physics) -> (Coast, Vec<Heard>) {
    let v0 = track.release(now, physics);
    match v0 == (0.0, 0.0) {
        true => (Coast::Idle, vec![heard(GesturePhase::Ended, by)]),
        false => (
            Coast::Gliding(Gliding {
                v0,
                since: now,
                heard: (0.0, 0.0),
            }),
            (by != (0.0, 0.0))
                .then(|| heard(GesturePhase::Changed, by))
                .into_iter()
                .collect(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blitz_kit::scroll::config::ScrollSettings;
    use std::time::Duration;

    fn at(ms: u64) -> Elapsed {
        Elapsed(Duration::from_millis(ms))
    }

    fn physics() -> Physics {
        Physics::from_settings(&ScrollSettings::default())
    }

    fn phases(heard: &[Heard]) -> Vec<GesturePhase> {
        heard.iter().map(|h| h.phase).collect()
    }

    /// A flick: fingers down at 0 ms, moving `step` px (y) every 8 ms for `moves` moves, lifting
    /// with no last motion. The state after the lift, and everything heard.
    fn flick(step: f64, moves: u64, physics: &Physics) -> (Coast, Vec<Heard>, Elapsed) {
        let mut coast = Coast::Idle;
        let mut said = Vec::new();
        for n in 0..moves {
            let phase = match n {
                0 => GesturePhase::Began,
                _ => GesturePhase::Changed,
            };
            let (next, heard) = coast.touch(phase, (0.0, step), at(n * 8), physics);
            coast = next;
            said.extend(heard);
        }
        let lifted = at(moves * 8);
        let (next, heard) = coast.touch(GesturePhase::Ended, (0.0, 0.0), lifted, physics);
        said.extend(heard);
        (next, said, lifted)
    }

    #[test]
    fn the_fingers_motion_passes_through_with_its_phases() {
        let physics = physics();
        let (_, said, _) = flick(0.0, 1, &physics);
        assert_eq!(
            said,
            vec![
                heard(GesturePhase::Began, (0.0, 0.0)),
                heard(GesturePhase::Ended, (0.0, 0.0))
            ],
            "a touch that never moved ends at once, with no glide"
        );
        let (coast, said, _) = flick(30.0, 6, &physics);
        assert_eq!(
            phases(&said),
            [GesturePhase::Began]
                .into_iter()
                .chain([GesturePhase::Changed; 5])
                .collect::<Vec<_>>(),
            "the lift of a fast flick is not heard yet"
        );
        assert!(matches!(coast, Coast::Gliding(_)), "{coast:?}");
        assert_eq!(
            said[1].by,
            (0.0, 30.0),
            "the fingers' px are heard unchanged"
        );
    }

    #[test]
    fn a_flick_glides_the_engines_distance_then_ends_once() {
        let physics = physics();
        let (mut coast, _, lifted) = flick(30.0, 6, &physics);
        let Coast::Gliding(glide) = coast.clone() else {
            panic!("a fast flick glides: {coast:?}");
        };
        let expected = physics.glide.total(glide.v0.1.abs());
        let mut sum = 0.0;
        let mut ends = Vec::new();
        for frame in 1..=400 {
            let (next, heard) = coast.advance(
                Elapsed(lifted.0 + Duration::from_millis(frame * 8)),
                &physics,
            );
            coast = next;
            for h in heard {
                sum += h.by.1;
                if h.phase == GesturePhase::Ended {
                    ends.push(frame);
                }
            }
        }
        assert!(
            (sum - expected).abs() < 1e-6 && expected > 100.0,
            "the frames sum to the engine's glide: {sum} vs {expected}"
        );
        assert_eq!(ends.len(), 1, "Ended is heard once: {ends:?}");
        assert_eq!(coast, Coast::Idle);
    }

    #[test]
    fn a_slow_lift_does_not_glide_and_ends_with_its_last_motion() {
        let physics = physics();
        let mut coast = Coast::Idle;
        for n in 0..4 {
            let phase = if n == 0 {
                GesturePhase::Began
            } else {
                GesturePhase::Changed
            };
            coast = coast.touch(phase, (0.0, 0.5), at(n * 40), &physics).0;
        }
        let (coast, heard) = coast.touch(GesturePhase::Ended, (0.0, 0.5), at(160), &physics);
        assert_eq!(heard, vec![self::heard(GesturePhase::Ended, (0.0, 0.5))]);
        assert_eq!(coast, Coast::Idle);
    }

    #[test]
    fn a_touch_during_the_glide_ends_it_before_the_new_run_begins() {
        let physics = physics();
        let (coast, _, lifted) = flick(30.0, 6, &physics);
        let (coast, heard) = coast.touch(
            GesturePhase::Began,
            (0.0, 1.0),
            Elapsed(lifted.0 + Duration::from_millis(40)),
            &physics,
        );
        assert_eq!(phases(&heard), [GesturePhase::Ended, GesturePhase::Began]);
        assert!(matches!(coast, Coast::Down(_)));
    }

    #[test]
    fn a_cancelled_run_stops_without_a_glide() {
        let physics = physics();
        let (coast, _, _) = flick(30.0, 5, &physics);
        let (coast, _) = coast.touch(GesturePhase::Cancelled, (0.0, 0.0), at(100), &physics);
        assert_eq!(coast, Coast::Idle);
        let down = Coast::Idle
            .touch(GesturePhase::Began, (0.0, 30.0), at(0), &physics)
            .0;
        let (idle, heard) = down.touch(GesturePhase::Cancelled, (0.0, 0.0), at(8), &physics);
        assert_eq!(
            (idle, phases(&heard)),
            (Coast::Idle, vec![GesturePhase::Cancelled])
        );
    }

    #[test]
    fn momentum_off_never_glides() {
        let off = Physics::from_settings(&ScrollSettings {
            momentum: ScrollMomentum::Off,
            ..ScrollSettings::default()
        });
        let (coast, said, _) = flick(30.0, 6, &off);
        assert_eq!(coast, Coast::Idle);
        assert_eq!(
            said.last().map(|h| h.phase),
            Some(GesturePhase::Ended),
            "the lift is heard at once"
        );
    }
}
