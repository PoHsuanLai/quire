//! A spring described the way [FLUID] and SwiftUI describe one (design/05-MOTION.md section 14):
//! a damping ratio and a response, not a duration. A spring has no length; it ends when it comes
//! to rest, and a new target mid-flight starts from where it is at the speed it has.
//!
//! This file is the vocabulary and the closed-form motion of one leg, as pure arithmetic; the
//! frame driver is `use_spring`.

use std::time::Duration;

/// A ratio in thousandths: 1000 is 1.0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Ratio(pub u16);

impl Ratio {
    /// Critical damping, 1.0: the fastest approach with no overshoot ([FLUID]'s default).
    pub const CRITICAL: Ratio = Ratio(1000);
    /// 0.8: a little bounce, for a gesture that had momentum toward its target ([FLUID]).
    pub const MOMENTUM: Ratio = Ratio(800);

    /// As a float, held to 0.05..=1.0 (a spring never over-damps, and never rings forever).
    fn get(self) -> f64 {
        f64::from(self.0.clamp(50, 1000)) / 1000.0
    }
}

/// A span of time in whole milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Millis(pub u32);

impl Millis {
    /// As a `Duration`.
    pub fn duration(self) -> Duration {
        Duration::from_millis(u64::from(self.0))
    }
}

/// A spring: how much it is damped (1.0 critical, below that it overshoots) and its response,
/// the period it would swing at undamped, which reads as how quick it feels. Only
/// [`crate::motion::spring_spec::SpringSpec`] makes one, so no caller picks a raw damping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Spring {
    damping: Ratio,
    response: Millis,
}

impl Spring {
    pub(crate) const fn new(damping: Ratio, response: Millis) -> Spring {
        Spring { damping, response }
    }

    /// Its damping ratio.
    pub fn damping(self) -> Ratio {
        self.damping
    }

    /// Its response.
    pub fn response(self) -> Millis {
        self.response
    }

    /// The undamped angular frequency, per second.
    fn omega(self) -> f64 {
        let seconds = f64::from(self.response.0.max(1)) / 1000.0;
        std::f64::consts::TAU / seconds
    }
}

/// Where a spring is and how fast it moves, in its caller's units and units per second.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct State {
    /// The position.
    pub position: f64,
    /// The velocity, units per second.
    pub velocity: f64,
}

/// One leg of a spring's motion: from `start` toward `target` under `spring`. A retarget ends
/// the leg and starts another from the state the old one has at that instant, which is what
/// keeps position and velocity continuous (design/27 section 3.12 rule 4).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Leg {
    /// Where the leg began and how fast.
    pub start: State,
    /// Where it is going.
    pub target: f64,
    /// Under what spring.
    pub spring: Spring,
}

/// How close to its target, in pixels, a spring must be to rest.
pub(crate) const REST_PX: f64 = 0.25;

/// How slow, in pixels per second, a spring must be to rest.
pub(crate) const REST_PX_PER_S: f64 = 4.0;

/// Whether a spring is still moving.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpringPhase {
    /// On its way; it asks for frames.
    Moving,
    /// At its target and still; it asks for nothing (design/26 R3).
    Rest,
}

impl Leg {
    /// A leg standing still at `at`.
    pub fn still(at: f64, spring: Spring) -> Leg {
        Leg {
            start: State {
                position: at,
                velocity: 0.0,
            },
            target: at,
            spring,
        }
    }

    /// The state `elapsed` into the leg, in closed form (no step error, so the state at any
    /// instant is exact however the frames fall).
    pub fn at(self, elapsed: Duration) -> State {
        let t = elapsed.as_secs_f64();
        let omega = self.spring.omega();
        let zeta = self.spring.damping.get();
        let d0 = self.start.position - self.target;
        let v0 = self.start.velocity;
        let (d, v) = if zeta >= 1.0 {
            critical(d0, v0, omega, t)
        } else {
            under(d0, v0, omega, zeta, t)
        };
        State {
            position: self.target + d,
            velocity: v,
        }
    }

    /// Whether the leg has come to rest by `elapsed`, `px_per_unit` saying how many pixels one
    /// of its units draws.
    pub fn phase(self, elapsed: Duration, px_per_unit: f64) -> SpringPhase {
        let now = self.at(elapsed);
        let near = ((now.position - self.target) * px_per_unit).abs() < REST_PX;
        let slow = (now.velocity * px_per_unit).abs() < REST_PX_PER_S;
        match (near, slow) {
            (true, true) => SpringPhase::Rest,
            _ => SpringPhase::Moving,
        }
    }
}

/// Critically damped: `d(t) = (A + B t) e^(-w t)`.
fn critical(d0: f64, v0: f64, omega: f64, t: f64) -> (f64, f64) {
    let b = v0 + omega * d0;
    let decay = (-omega * t).exp();
    let d = (d0 + b * t) * decay;
    let v = (b - omega * (d0 + b * t)) * decay;
    (d, v)
}

/// Under-damped: `d(t) = e^(-z w t) (A cos(wd t) + B sin(wd t))`.
fn under(d0: f64, v0: f64, omega: f64, zeta: f64, t: f64) -> (f64, f64) {
    let wd = omega * (1.0 - zeta * zeta).sqrt();
    let a = d0;
    let b = (v0 + zeta * omega * d0) / wd;
    let decay = (-zeta * omega * t).exp();
    let (sin, cos) = (wd * t).sin_cos();
    let d = decay * (a * cos + b * sin);
    let v = decay * ((-zeta * omega * a + wd * b) * cos + (-zeta * omega * b - wd * a) * sin);
    (d, v)
}

#[cfg(test)]
#[path = "spring_tests.rs"]
mod tests;
