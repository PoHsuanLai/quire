//! A spring as a timeline (design/05-MOTION.md section 14): one leg of its closed-form motion,
//! which has no length of its own and ends when it comes to rest. Its frame is where to draw
//! it: a position, a velocity, and whether it is still moving.

use super::Timeline;
use crate::spring::{Leg, SpringPhase, SpringTuning, State};
use ds_core::time::FRAME_TICK;
use std::time::Duration;

/// The longest a spring is looked at for its rest: past it, a spring counts as resting (the
/// slowest damping a spring may have, 0.05, rings out well inside it).
const LONGEST: Duration = Duration::from_secs(30);

/// How many logical pixels one of a spring's units draws: 1 for a spring in pixels, a segment's
/// width for one counted in segments. It scales a hand's velocity into the spring's units and
/// decides when the spring is close enough to rest.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PxPerUnit(pub f32);

impl Default for PxPerUnit {
    fn default() -> Self {
        PxPerUnit(1.0)
    }
}

impl PxPerUnit {
    /// As the float the closed form multiplies by.
    pub(crate) fn get(self) -> f64 {
        f64::from(self.0.max(f32::MIN_POSITIVE))
    }
}

/// A spring's frame: where to draw it now.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringFrame {
    position: f32,
    velocity: f32,
    phase: SpringPhase,
}

impl SpringFrame {
    /// A frame at `state`, `phase` along.
    pub(crate) fn of(state: State, phase: SpringPhase) -> SpringFrame {
        SpringFrame {
            position: state.position as f32,
            velocity: state.velocity as f32,
            phase,
        }
    }

    /// The position, in the spring's units (past the target on the way when it overshoots).
    pub fn position(self) -> f32 {
        self.position
    }

    /// The velocity, units per second.
    pub fn velocity(self) -> f32 {
        self.velocity
    }

    /// Moving or at rest.
    pub fn phase(self) -> SpringPhase {
        self.phase
    }

    /// The position for a stylesheet: three decimals, `0.125`.
    pub fn css(self) -> String {
        format!("{:.3}", self.position)
    }
}

/// One leg of a spring's motion, in units that draw `scale` pixels each.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spring {
    leg: Leg,
    scale: PxPerUnit,
}

impl Spring {
    /// The spring that runs `leg`.
    pub fn new(leg: Leg, scale: PxPerUnit) -> Spring {
        Spring { leg, scale }
    }

    /// A spring standing at `at`, on `tuning`.
    pub fn still(at: f64, tuning: SpringTuning, scale: PxPerUnit) -> Spring {
        Spring::new(Leg::still(at, tuning), scale)
    }

    /// The leg it runs.
    pub fn leg(&self) -> Leg {
        self.leg
    }

    /// The exact state `elapsed` into the leg, between frames too.
    pub fn state(&self, elapsed: Duration) -> State {
        self.leg.at(elapsed)
    }

    /// Whether the leg has come to rest by `elapsed`.
    fn phase(&self, elapsed: Duration) -> SpringPhase {
        self.leg.phase(elapsed, self.scale.get())
    }
}

impl Timeline for Spring {
    type Frame = SpringFrame;

    /// The first frame the leg is at rest, looked for a frame at a time.
    fn total(&self) -> Duration {
        std::iter::successors(Some(Duration::ZERO), |at| Some(*at + FRAME_TICK))
            .take_while(|at| *at < LONGEST)
            .find(|at| self.settled(*at))
            .unwrap_or(LONGEST)
    }

    /// The leg's state while it moves; exactly its target, still, once it rests.
    fn at(&self, elapsed: Duration) -> SpringFrame {
        match self.phase(elapsed) {
            SpringPhase::Moving => SpringFrame::of(self.leg.at(elapsed), SpringPhase::Moving),
            SpringPhase::Rest => SpringFrame::of(
                State {
                    position: self.leg.target,
                    velocity: 0.0,
                },
                SpringPhase::Rest,
            ),
        }
    }

    /// A spring has no length: it is settled when it rests.
    fn settled(&self, elapsed: Duration) -> bool {
        self.phase(elapsed) == SpringPhase::Rest
    }
}

#[cfg(test)]
mod tests {
    use super::{PxPerUnit, Spring, SpringFrame, Timeline};
    use crate::spring::{Leg, Millis, Ratio, SpringPhase, SpringTuning, State};
    use std::time::Duration;

    fn from_to(from: f64, to: f64) -> Spring {
        Spring::new(
            Leg {
                start: State {
                    position: from,
                    velocity: 0.0,
                },
                target: to,
                spring: SpringTuning::new(Ratio::CRITICAL, Millis(450)),
            },
            PxPerUnit(1.0),
        )
    }

    #[test]
    fn a_spring_moves_from_its_start_and_rests_exactly_on_its_target() {
        let spring = from_to(0.0, 100.0);
        let total = spring.total();
        assert!(
            (300..1_200).contains(&(total.as_millis() as u64)),
            "rests at {total:?}"
        );
        let rest = SpringFrame::of(
            State {
                position: 100.0,
                velocity: 0.0,
            },
            SpringPhase::Rest,
        );
        assert_eq!(spring.at(total), rest);
        assert_eq!(spring.at(total * 4), rest);
        let start = spring.at(Duration::ZERO);
        assert_eq!(
            (start.position(), start.phase()),
            (0.0, SpringPhase::Moving)
        );
        let half = spring.at(total / 2);
        assert!(
            (1.0..100.0).contains(&half.position()) && half.phase() == SpringPhase::Moving,
            "{half:?}"
        );
    }

    #[test]
    fn a_spring_standing_still_is_settled_at_once() {
        let still = Spring::still(
            42.0,
            SpringTuning::new(Ratio::CRITICAL, Millis(300)),
            PxPerUnit(1.0),
        );
        assert_eq!(still.total(), Duration::ZERO);
        assert!(still.settled(Duration::ZERO));
        assert_eq!(still.at(Duration::ZERO).position(), 42.0);
    }
}
