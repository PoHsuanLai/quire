//! The spring driver (design/05-MOTION.md section 14): a [`Spring`] played on a `Playback`, on
//! this thread's frame clock (`ds::time`), asking for a frame every `FRAME_TICK` while it moves
//! and nothing once it rests (design/26 R3). A new target mid-flight starts a new leg from the
//! exact state the old one has at that instant, so position and velocity never jump; a hand's
//! release hands its velocity to the leg (design/27 section 3.12).

use super::spring::{Leg, SpringPhase, State};
use super::spring_spec::SpringSpec;
use super::timeline::playback::{Playback, use_playback};
use super::timeline::spring::{PxPerUnit, Spring, SpringFrame};
use dioxus::core::queue_effect;
use dioxus::prelude::*;
use ds_style::appearance::motion::MotionLevel;
use ds_style::scope::Scope;

/// A spring a component drives: read its frame in render, move it from a handler or an effect.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringMotion {
    playback: Playback<Spring>,
    env: Option<Signal<Scope>>,
    scale: PxPerUnit,
}

impl SpringMotion {
    /// The frame now, subscribing the caller's render to the spring's frames.
    pub fn frame(self) -> SpringFrame {
        self.playback.frame()
    }

    /// The frame now, without subscribing.
    pub fn peek(self) -> SpringFrame {
        self.playback
            .peek()
            .unwrap_or_else(|| SpringFrame::of(State::default(), SpringPhase::Rest))
    }

    /// Where the spring is this instant on the frame clock, exactly (between frames too).
    pub fn state(self) -> State {
        self.playback.timeline().map_or(State::default(), |spring| {
            spring.state(self.playback.elapsed())
        })
    }

    /// Move toward `target` on the spring `spec` chooses: from where it is at the speed it has,
    /// or at the contact's release velocity when the touch carries one. Call from a handler or
    /// an effect.
    pub fn go(self, target: f32, spec: SpringSpec) {
        let target = f64::from(target);
        let now = self.state();
        let scale = self.scale.get();
        let level = self.level();
        let thrown = spec.thrown(level).px_per_s() / scale;
        let velocity = if thrown == 0.0 { now.velocity } else { thrown };
        let leg = Leg {
            start: State {
                position: now.position,
                velocity,
            },
            target,
            spring: spec.spring((target - now.position) * scale, level),
        };
        self.playback.play(Spring::new(leg, self.scale));
    }

    /// Stand at `at`, following a hand 1:1: no spring and no frames of its own.
    pub fn track(self, at: f32) {
        if let Some(spring) = self.playback.timeline() {
            let tuning = spring.leg().spring;
            self.playback
                .play(Spring::still(f64::from(at), tuning, self.scale));
        }
    }

    /// Stand at `at` at once.
    pub fn snap(self, at: f32) {
        self.track(at);
    }

    fn level(self) -> MotionLevel {
        self.env
            .and_then(|env| env.try_peek().ok().map(|env| env.resolved.motion))
            .unwrap_or(MotionLevel::Standard)
    }
}

/// A spring standing at `at`, in units of which one draws `scale` pixels. Move it with
/// [`SpringMotion::go`] and [`SpringMotion::track`].
pub fn use_spring_motion(at: f32, scale: PxPerUnit) -> SpringMotion {
    let resting = SpringSpec::for_touch(crate::detail::touch::Touch::Remote)
        .spring(0.0, MotionLevel::Standard);
    SpringMotion {
        playback: use_playback(Spring::still(f64::from(at), resting, scale)),
        env: try_use_context::<Signal<Scope>>(),
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
