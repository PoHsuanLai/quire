//! A level that sweeps to its value from Rust, one recomputed frame at a time
//! (design/23-WIDGETS.md section 4.1; design/05-MOTION.md section 4.12).
//!
//! Blitz's stylesheet cannot reach inside an SVG, so a ring whose arc must grow cannot be
//! animated by a keyframe: the path itself has to change. As an animated emoji moves its frames from
//! Rust timers, this hook plays a [`Sweep`] on a playback, whose task wakes every `FRAME_TICK`,
//! computes the frame and writes it only when it differs, and ends when the sweep and its tail
//! are done: a sweep at rest runs no task and paints 0 frames (the idle-frame rule). The task
//! belongs to the hook's owner and is dropped with it.
//!
//! On mount and on each new [`WakeStamp`] the level sweeps from empty; on a new level it sweeps
//! from what it drew last (the old level, or wherever a running sweep had reached). Under
//! Reduced motion nothing sweeps: every frame is the final one.

use crate::core::task::{Gone, try_get};
use crate::core::vocab::Fraction;
use crate::motion::timeline::playback::{Playback, use_playback};
use crate::motion::timeline::sweep::{RunFrame, RunTail, RunTokens, Sweep};
use crate::motion::wake::WakeStamp;
use crate::style::appearance::motion::MotionLevel;
use crate::style::scope::{Scope, use_scope_signal};
use dioxus::core::queue_effect;
use dioxus::prelude::*;

/// Where a new sweep starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Origin {
    /// Empty: a mount or a wake.
    Empty,
    /// What was drawn last: a change of level.
    Drawn,
}

/// The frame of a level sweeping to `level` with `tokens`: from empty on mount and on each new
/// `wake`, from the level drawn last on each new `level`. Re-renders its owner once per
/// changed frame while it sweeps and never at rest. Needs an enclosing `Ds`.
pub fn use_level_run(level: Fraction, wake: WakeStamp, tokens: RunTokens) -> RunFrame {
    let env = use_scope_signal();
    let playback = use_playback(from_empty(env.peek().resolved.motion, level, tokens));
    // The last level and stamp seen live in a plain value: a signal written while rendering
    // would schedule a second render for nothing (as `use_bump_on`).
    let mut seen = use_hook(|| CopyValue::new(None::<(Fraction, WakeStamp)>));
    let before = *seen.peek();
    if before != Some((level, wake)) {
        seen.set(Some((level, wake)));
        let origin = match before {
            Some((_, was)) if was == wake => Origin::Drawn,
            Some(_) | None => Origin::Empty,
        };
        queue_effect(move || {
            let _ = start(playback, env, level, origin, tokens);
        });
    }
    playback.frame()
}

/// The sweep from empty to `level` at `motion`: standing at the level under Reduced motion.
fn from_empty(motion: MotionLevel, level: Fraction, tokens: RunTokens) -> Sweep {
    match motion {
        MotionLevel::Reduced => Sweep::at_rest(level),
        MotionLevel::Calm | MotionLevel::Standard | MotionLevel::Extra => Sweep {
            from: Fraction(0),
            to: level,
            tail: RunTail::Follows,
            timing: tokens.timing(motion),
        },
    }
}

/// A change's tail: whole if the last frame's was, else it still follows (a change landing
/// mid-entrance keeps the entrance's promise to fade the tail in at the end).
fn tail_after(drawn: RunFrame) -> RunTail {
    if drawn.tail.0 >= 1000 {
        RunTail::Stays
    } else {
        RunTail::Follows
    }
}

/// Stop the running sweep and start one to `level`.
fn start(
    playback: Playback<Sweep>,
    env: Signal<Scope>,
    level: Fraction,
    origin: Origin,
    tokens: RunTokens,
) -> Result<(), Gone> {
    let motion = try_get(env)?.resolved.motion;
    let drawn = playback.peek().ok_or(Gone)?;
    let sweep = match origin {
        Origin::Empty => from_empty(motion, level, tokens),
        Origin::Drawn => match motion {
            MotionLevel::Reduced => Sweep::at_rest(level),
            MotionLevel::Calm | MotionLevel::Standard | MotionLevel::Extra => Sweep {
                from: drawn.shown,
                to: level,
                tail: tail_after(drawn),
                timing: tokens.timing(motion),
            },
        },
    };
    playback.play(sweep);
    Ok(())
}
