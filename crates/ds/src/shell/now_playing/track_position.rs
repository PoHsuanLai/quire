//! TrackPosition: the Now Playing position bar (design/26-DETAILS.md 5.2.10). A clock-paced
//! report steps, it does not tween (3.1 Progress): while playing, the bar and the times repaint
//! once a second, on the second, from the last position reported, and never between; paused or
//! buffering, the bar holds at 0 frames. Mounted only while the panel is open, so it costs one
//! frame a second only while someone can see it.

use crate::core::task::{Gone, spawn_in, try_get, try_set};
use crate::core::time::clock::sleep;
use crate::shell::now_playing::kind::{Playback, PositionClock};
use dioxus::core::{Task, current_scope_id, queue_effect};
use dioxus::prelude::*;
use std::time::{Duration, Instant};

/// A position report: where, out of how long, and whether it runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Report {
    at: Duration,
    length: Duration,
    clock: PositionClock,
}

/// The last report and when it arrived, and the ticker that repaints it.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Ticker {
    origin: Signal<(Report, Instant)>,
    beats: Signal<u32>,
    task: Signal<Option<Task>>,
    scope: ScopeId,
}

impl Ticker {
    /// Restart the ticker for `report`, which arrived now.
    fn restart(self, report: Report) -> Result<(), Gone> {
        if let Some(running) = try_get(self.task)? {
            running.cancel();
        }
        try_set(self.origin, (report, crate::core::time::clock::now()))?;
        try_set(self.task, None)?;
        if report.clock == PositionClock::Held {
            return Ok(());
        }
        let task = spawn_in(self.scope, async move {
            let _ = self.run().await;
        });
        try_set(self.task, Some(task))
    }

    /// Wake on each whole second of the track until its end.
    async fn run(self) -> Result<(), Gone> {
        loop {
            let (report, since) = try_get(self.origin)?;
            let now = position(report, crate::core::time::clock::since(since));
            if now >= report.length {
                return Ok(());
            }
            sleep(to_next_second(now)).await;
            let beat = try_get(self.beats)?;
            try_set(self.beats, beat.wrapping_add(1))?;
        }
    }
}

/// Where the track is `elapsed` after `report` arrived: running from it, or held at it, never
/// past the end.
fn position(report: Report, elapsed: Duration) -> Duration {
    let at = match report.clock {
        PositionClock::Running => report.at + elapsed,
        PositionClock::Held => report.at,
    };
    at.min(report.length)
}

/// How long until the position reaches its next whole second.
fn to_next_second(at: Duration) -> Duration {
    let into = u64::from(at.subsec_millis());
    Duration::from_millis(1000 - into)
}

/// `m:ss`, or `h:mm:ss` past an hour.
fn clock_text(at: Duration) -> String {
    let seconds = at.as_secs();
    let (hours, minutes, seconds) = (seconds / 3600, seconds / 60 % 60, seconds % 60);
    match hours {
        0 => format!("{minutes}:{seconds:02}"),
        _ => format!("{hours}:{minutes:02}:{seconds:02}"),
    }
}

/// The track at `at` of `length` (the player's last report), advancing a second at a time while
/// `playback` is `Playing`. A new report re-anchors it; the same report again changes nothing.
#[component]
pub fn TrackPosition(
    at: Duration,
    length: Duration,
    #[props(default)] playback: Playback,
) -> Element {
    let report = Report {
        at: at.min(length),
        length,
        clock: playback.clock(),
    };
    let ticker = Ticker {
        origin: use_signal(|| (report, crate::core::time::clock::now())),
        beats: use_signal(|| 0),
        task: use_signal(|| None),
        scope: use_hook(current_scope_id),
    };
    let mut seen = use_hook(|| CopyValue::new(None::<Report>));
    if *seen.peek() != Some(report) {
        seen.set(Some(report));
        queue_effect(move || {
            let _ = ticker.restart(report);
        });
    }
    // Subscribe to the beat: each whole second repaints.
    let _ = (ticker.beats)();
    let (anchored, since) = *ticker.origin.peek();
    let shown = if anchored == report {
        position(report, crate::core::time::clock::since(since))
    } else {
        report.at
    };
    let whole = Duration::from_secs(shown.as_secs());
    let share = match length.as_millis() {
        0 => 0.0,
        total => whole.as_millis() as f64 / total as f64,
    };
    let left = length.saturating_sub(whole);
    let (elapsed, remaining) = (clock_text(whole), clock_text(left));
    rsx! {
        div {
            class: "ds-track-position",
            role: "progressbar",
            "aria-label": "Position",
            "aria-valuemin": "0",
            "aria-valuemax": "{length.as_secs()}",
            "aria-valuenow": "{whole.as_secs()}",
            "aria-valuetext": "{elapsed} of {clock_text(length)}",
            span { class: "ds-track-bar", style: "--f:{share:.4}",
                span { class: "ds-track-bar-fill" }
            }
            span { class: "ds-track-times",
                span { class: "ds-track-time", "{elapsed}" }
                span { class: "ds-track-time", "-{remaining}" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Report, clock_text, position, to_next_second};
    use crate::shell::now_playing::kind::PositionClock;
    use std::time::Duration;

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    #[test]
    fn a_running_position_advances_from_its_report_and_stops_at_the_end() {
        let running = Report {
            at: ms(61_400),
            length: ms(200_000),
            clock: PositionClock::Running,
        };
        let held = Report {
            clock: PositionClock::Held,
            ..running
        };
        let cases = [
            (running, ms(0), ms(61_400)),
            (running, ms(600), ms(62_000)),
            (running, ms(500_000), ms(200_000)),
            (held, ms(5_000), ms(61_400)),
        ];
        for (report, elapsed, want) in cases {
            assert_eq!(position(report, elapsed), want, "{report:?} +{elapsed:?}");
        }
    }

    #[test]
    fn it_wakes_on_the_second_and_writes_clock_time() {
        assert_eq!(to_next_second(ms(61_400)), ms(600));
        assert_eq!(to_next_second(ms(62_000)), ms(1_000));
        let cases = [
            (0, "0:00"),
            (61, "1:01"),
            (600, "10:00"),
            (3_725, "1:02:05"),
        ];
        for (seconds, want) in cases {
            assert_eq!(clock_text(Duration::from_secs(seconds)), want);
        }
    }
}
