//! The frame driver under every Rust-played [`Timeline`]: a run advanced by a task that asks for
//! a frame every `FRAME_TICK` while it moves and stops when it settles (R3), starting a new run
//! from where the caller says (R10). [`use_playback`] is the handle a component moves by hand.

use super::Timeline;
use crate::core::task::{Gone, spawn_in, try_get, try_set, try_set_if_changed};
use crate::core::time::{FRAME_TICK, clock};
use dioxus::core::{Task, current_scope_id};
use dioxus::prelude::*;
use std::time::{Duration, Instant};

/// The run in progress: its timeline, when it started and which run it is.
#[derive(Debug, Clone, PartialEq)]
struct Run<T> {
    timeline: T,
    started: Instant,
    serial: u32,
}

/// A timeline and the task that advances it.
pub(crate) struct Playback<T: Timeline> {
    run: Signal<Run<T>>,
    frame: Signal<T::Frame>,
    task: Signal<Option<Task>>,
    scope: ScopeId,
}

impl<T: Timeline> Clone for Playback<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Timeline> Copy for Playback<T> {}

impl<T: Timeline> Playback<T> {
    /// The frame now, subscribing the caller's render to the run's frames.
    pub(crate) fn frame(self) -> T::Frame {
        (self.frame)()
    }

    /// The frame last drawn, without subscribing; `None` once the owner has gone.
    pub(crate) fn peek(self) -> Option<T::Frame> {
        self.frame.try_peek().ok().map(|frame| frame.clone())
    }

    /// The timeline now running (or last run), without subscribing; `None` once the owner has
    /// gone.
    pub(crate) fn timeline(self) -> Option<T> {
        self.run.try_peek().ok().map(|run| run.timeline.clone())
    }

    /// Which run this is: every [`Self::play`] is a new one, so a reader can tell a move it
    /// armed for from an earlier one.
    pub(crate) fn serial(self) -> u32 {
        self.run.try_peek().map_or(0, |run| run.serial)
    }

    /// Play `timeline` from its start; a run already going stops where it is. Call from an
    /// effect or a handler.
    pub(crate) fn play(self, timeline: T) {
        self.play_from(timeline, Duration::ZERO);
    }

    /// Play `timeline` as if it had started `already` ago.
    pub(crate) fn play_from(self, timeline: T, already: Duration) {
        let _ = self.try_play(timeline, already);
    }

    fn try_play(self, timeline: T, already: Duration) -> Result<(), Gone> {
        if let Some(running) = try_get(self.task)? {
            running.cancel();
        }
        try_set(self.task, None)?;
        let now = clock::now();
        let started = now.checked_sub(already).unwrap_or(now);
        let serial = try_get(self.run)?.serial.wrapping_add(1);
        try_set(
            self.run,
            Run {
                timeline: timeline.clone(),
                started,
                serial,
            },
        )?;
        try_set_if_changed(self.frame, timeline.at(already))?;
        if timeline.settled(already) {
            return Ok(());
        }
        let task = spawn_in(self.scope, async move {
            let _ = self.advance(timeline, started).await;
        });
        try_set(self.task, Some(task))
    }

    async fn advance(self, timeline: T, started: Instant) -> Result<(), Gone> {
        tick(&timeline, started, |frame| {
            try_set_if_changed(self.frame, frame)
        })
        .await?;
        try_set(self.task, None)
    }
}

/// A playback standing at the frame `timeline` starts with, not yet running.
pub(crate) fn use_playback<T: Timeline>(timeline: T) -> Playback<T> {
    let first = timeline.at(Duration::ZERO);
    Playback {
        run: use_signal(|| Run {
            timeline,
            started: clock::now(),
            serial: 0,
        }),
        frame: use_signal(|| first),
        task: use_signal(|| None),
        scope: use_hook(current_scope_id),
    }
}

/// Hand `emit` each frame of `timeline` from a frame `FRAME_TICK` after `started`, and the
/// last one when it settles. Ends early with [`Gone`] when `emit` says its target has.
pub(crate) async fn tick<T: Timeline>(
    timeline: &T,
    started: Instant,
    mut emit: impl FnMut(T::Frame) -> Result<(), Gone>,
) -> Result<(), Gone> {
    loop {
        clock::sleep(FRAME_TICK).await;
        let elapsed = clock::since(started);
        emit(timeline.at(elapsed))?;
        if timeline.settled(elapsed) {
            return Ok(());
        }
    }
}
