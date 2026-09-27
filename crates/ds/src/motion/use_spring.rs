//! The spring driver (design/05-MOTION.md section 14): a spring advanced on this thread's frame
//! clock (`ds::time`), asking for a frame every [`FRAME_TICK`] while it moves and nothing once it
//! rests (design/26 R3). A new target mid-flight starts a new leg from the exact state the old
//! one has at that instant, so position and velocity never jump; a hand's release hands its
//! velocity to the leg (design/27 section 3.12).

use super::spring::{Leg, Spring, SpringPhase, State};
use super::spring_spec::SpringSpec;
use crate::root::env::Env;
use crate::task::{Gone, spawn_in, try_get, try_set};
use crate::time::{FRAME_TICK, sleep};
use dioxus::core::{Task, current_scope_id, queue_effect};
use dioxus::prelude::*;
use std::time::Instant;

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
    fn get(self) -> f64 {
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
    fn of(state: State, phase: SpringPhase) -> SpringFrame {
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

/// The leg a spring is on and when it began.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Run {
    leg: Leg,
    started: Instant,
}

/// A spring a component drives: read its frame in render, move it from a handler or an effect.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringMotion {
    run: Signal<Run>,
    frame: Signal<SpringFrame>,
    task: Signal<Option<Task>>,
    env: Option<Signal<Env>>,
    scope: ScopeId,
    scale: PxPerUnit,
}

impl SpringMotion {
    /// The frame now, subscribing the caller's render to the spring's frames.
    pub fn frame(self) -> SpringFrame {
        (self.frame)()
    }

    /// The frame now, without subscribing.
    pub fn peek(self) -> SpringFrame {
        self.frame
            .try_peek()
            .map_or(SpringFrame::of(State::default(), SpringPhase::Rest), |f| *f)
    }

    /// Where the spring is this instant on the frame clock, exactly (between frames too).
    pub fn state(self) -> State {
        self.run.try_peek().map_or(State::default(), |run| {
            run.leg.at(crate::time::since(run.started))
        })
    }

    /// Move toward `target` on the spring `spec` chooses: from where it is at the speed it has,
    /// or at the contact's release velocity when the touch carries one. Call from a handler or
    /// an effect.
    pub fn go(self, target: f32, spec: SpringSpec) {
        let _ = self.try_go(f64::from(target), spec);
    }

    /// Stand at `at`, following a hand 1:1: no spring and no frames of its own.
    pub fn track(self, at: f32) {
        let _ = self.try_still(f64::from(at));
    }

    /// Stand at `at` at once.
    pub fn snap(self, at: f32) {
        self.track(at);
    }

    fn level(self) -> crate::appearance::MotionLevel {
        self.env
            .and_then(|env| env.try_peek().ok().map(|env| env.resolved.motion))
            .unwrap_or(crate::appearance::MotionLevel::Standard)
    }

    fn try_still(self, at: f64) -> Result<(), Gone> {
        self.cancel()?;
        let spring = try_get(self.run)?.leg.spring;
        let leg = Leg::still(at, spring);
        try_set(
            self.run,
            Run {
                leg,
                started: crate::time::now(),
            },
        )?;
        try_set(self.frame, SpringFrame::of(leg.start, SpringPhase::Rest))
    }

    fn try_go(self, target: f64, spec: SpringSpec) -> Result<(), Gone> {
        let now = self.state();
        self.cancel()?;
        let scale = self.scale.get();
        let level = self.level();
        let thrown = spec.thrown(level).px_per_s() / scale;
        let velocity = if thrown == 0.0 { now.velocity } else { thrown };
        let spring: Spring = spec.spring((target - now.position) * scale, level);
        let leg = Leg {
            start: State {
                position: now.position,
                velocity,
            },
            target,
            spring,
        };
        let started = crate::time::now();
        try_set(self.run, Run { leg, started })?;
        let phase = leg.phase(std::time::Duration::ZERO, scale);
        let first = match phase {
            SpringPhase::Rest => SpringFrame::of(
                State {
                    position: target,
                    velocity: 0.0,
                },
                phase,
            ),
            SpringPhase::Moving => SpringFrame::of(leg.start, phase),
        };
        try_set(self.frame, first)?;
        if phase == SpringPhase::Rest {
            return Ok(());
        }
        let task = spawn_in(self.scope, async move {
            let _ = self.play(leg, started).await;
        });
        try_set(self.task, Some(task))
    }

    fn cancel(self) -> Result<(), Gone> {
        if let Some(running) = try_get(self.task)? {
            running.cancel();
        }
        try_set(self.task, None)
    }

    async fn play(self, leg: Leg, started: Instant) -> Result<(), Gone> {
        let scale = self.scale.get();
        loop {
            sleep(FRAME_TICK).await;
            let elapsed = crate::time::since(started);
            match leg.phase(elapsed, scale) {
                SpringPhase::Moving => {
                    try_set(
                        self.frame,
                        SpringFrame::of(leg.at(elapsed), SpringPhase::Moving),
                    )?;
                }
                SpringPhase::Rest => {
                    let rest = State {
                        position: leg.target,
                        velocity: 0.0,
                    };
                    try_set(self.frame, SpringFrame::of(rest, SpringPhase::Rest))?;
                    return try_set(self.task, None);
                }
            }
        }
    }
}

/// A spring standing at `at`, in units of which one draws `scale` pixels. Move it with
/// [`SpringMotion::go`] and [`SpringMotion::track`].
pub fn use_spring_motion(at: f32, scale: PxPerUnit) -> SpringMotion {
    let resting = Leg::still(
        f64::from(at),
        SpringSpec::for_touch(crate::detail::Touch::Remote)
            .spring(0.0, crate::appearance::MotionLevel::Standard),
    );
    SpringMotion {
        run: use_signal(|| Run {
            leg: resting,
            started: crate::time::now(),
        }),
        frame: use_signal(|| SpringFrame::of(resting.start, SpringPhase::Rest)),
        task: use_signal(|| None),
        env: try_use_context::<Signal<Env>>(),
        scope: use_hook(current_scope_id),
        scale,
    }
}

/// A spring that follows `target`: it stands there on mount, and each time `target` changes it
/// springs there from where it is, at the speed it has, on the spring `spec` chooses (critical
/// for a remote change or a tap, 0.8 for a throw toward it; critical under Reduced). Asks for
/// frames only while it moves.
pub fn use_spring(target: f32, spec: SpringSpec, scale: PxPerUnit) -> SpringFrame {
    let motion = use_spring_motion(target, scale);
    let mut seen = use_hook(|| CopyValue::new(target));
    if *seen.peek() != target {
        seen.set(target);
        queue_effect(move || motion.go(target, spec));
    }
    motion.frame()
}
