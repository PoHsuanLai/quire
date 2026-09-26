//! A level that sweeps to its value from Rust, one recomputed frame at a time
//! (design/23-WIDGETS.md section 4.1; design/05-MOTION.md section 4.12).
//!
//! Blitz's stylesheet cannot reach inside an SVG, so a ring whose arc must grow cannot be
//! animated by a keyframe: the path itself has to change. As the persona moves its parts from
//! Rust timers, this hook runs a task that wakes every [`FRAME_TICK`], computes the frame
//! ([`crate::motion::level_run::frame_at`]) and writes it only when it differs, and ends when the
//! sweep and its tail are done: a sweep at rest runs no task and paints 0 frames (the
//! idle-frame rule). The task belongs to the hook's owner and is dropped with it.
//!
//! On mount and on each new [`WakeStamp`] the level sweeps from empty; on a new level it sweeps
//! from what it drew last (the old level, or wherever a running sweep had reached). Under
//! Reduced motion nothing sweeps: every frame is the final one.

use crate::appearance::MotionLevel;
use crate::components::vocab::Fraction;
use crate::motion::level_run::{
    LevelRun, RunFrame, RunPhase, RunTail, RunTokens, frame_at, phase_at,
};
use crate::motion::wake::WakeStamp;
use crate::root::env::{Env, use_env_signal};
use crate::task::{Gone, spawn_in, try_get, try_set, try_set_if_changed};
use crate::time::{FRAME_TICK, sleep};
use dioxus::core::{Task, current_scope_id, queue_effect};
use dioxus::prelude::*;
use std::time::{Duration, Instant};

/// Where a new sweep starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Origin {
    /// Empty: a mount or a wake.
    Empty,
    /// What was drawn last: a change of level.
    Drawn,
}

/// The hook's state: the frame drawn, the task drawing it, and where to read the motion level.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Runner {
    frame: Signal<RunFrame>,
    task: Signal<Option<Task>>,
    env: Signal<Env>,
    scope: ScopeId,
}

/// The frame of a level sweeping to `level` with `tokens`: from empty on mount and on each new
/// `wake`, from the level drawn last on each new `level`. Re-renders its owner once per
/// changed frame while it sweeps and never at rest. Needs an enclosing `Ds`.
pub fn use_level_run(level: Fraction, wake: WakeStamp, tokens: RunTokens) -> RunFrame {
    let env = use_env_signal();
    let runner = Runner {
        frame: use_signal(|| first_frame(env.peek().resolved.motion, level)),
        task: use_signal(|| None),
        env,
        scope: use_hook(current_scope_id),
    };
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
            let _ = start(runner, level, origin, tokens);
        });
    }
    (runner.frame)()
}

/// The frame before the first effect runs: empty, or the level itself under Reduced motion.
fn first_frame(motion: MotionLevel, level: Fraction) -> RunFrame {
    match motion {
        MotionLevel::Reduced => RunFrame::rest(level),
        MotionLevel::Calm | MotionLevel::Standard | MotionLevel::Extra => {
            RunFrame::start(LevelRun {
                from: Fraction(0),
                to: level,
                tail: RunTail::Follows,
            })
        }
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
fn start(runner: Runner, level: Fraction, origin: Origin, tokens: RunTokens) -> Result<(), Gone> {
    if let Some(running) = try_get(runner.task)? {
        running.cancel();
        try_set(runner.task, None)?;
    }
    let motion = try_get(runner.env)?.resolved.motion;
    if motion == MotionLevel::Reduced {
        return try_set_if_changed(runner.frame, RunFrame::rest(level));
    }
    let drawn = try_get(runner.frame)?;
    let run = match origin {
        Origin::Empty => LevelRun {
            from: Fraction(0),
            to: level,
            tail: RunTail::Follows,
        },
        Origin::Drawn => LevelRun {
            from: drawn.shown,
            to: level,
            tail: tail_after(drawn),
        },
    };
    let timing = tokens.timing(motion);
    try_set_if_changed(runner.frame, frame_at(run, timing, Duration::ZERO))?;
    let started = Instant::now();
    let (frame, task) = (runner.frame, runner.task);
    let running = spawn_in(runner.scope, async move {
        loop {
            sleep(FRAME_TICK).await;
            let elapsed = started.elapsed();
            if try_set_if_changed(frame, frame_at(run, timing, elapsed)).is_err() {
                return;
            }
            if phase_at(run, timing, elapsed) == RunPhase::Done {
                break;
            }
        }
        let _ = try_set(task, None);
    });
    try_set(runner.task, Some(running))
}
