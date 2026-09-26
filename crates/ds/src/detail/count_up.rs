//! CountUp: an integer counting to a new value, in step with a sweep or on its own clock
//! (design/26-DETAILS.md section 3.2). Named so because `ds::Count` is the count badge.

use super::cue::Cue;
use super::glide::Glide;
use super::level::use_level;
use super::moment::Moment;
use super::motor::use_motor;
use super::sweep::Sweep;
use super::tween::Tween;
use crate::appearance::MotionLevel;
use crate::tokens::{DurationToken, EasingToken};
use dioxus::core::queue_effect;
use dioxus::prelude::*;
use std::time::Duration;

/// What paces a count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountPace {
    /// In step with a sweep: the number reads the arc's own progress, so it lands with it.
    InStep(Sweep),
    /// On its own clock (a readout with no arc), over this token with `--e-out`.
    Own(DurationToken),
}

/// The number to print this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CountUp {
    shown: i64,
}

impl CountUp {
    /// The number to print: never past the target, its text repainted at most every
    /// `--t-count-step`.
    pub fn shown(self) -> i64 {
        self.shown
    }
}

/// One count: from where to where, which move of its pace it waits for, and what it last showed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Counting {
    serial: u32,
    from: i64,
    to: i64,
    /// The pace's run when the count was set: a later run is this count's move.
    armed: u32,
    /// The last frame's repaint bucket and number.
    last: Option<(u32, u64, i64)>,
}

/// Count to `value` as `cue` says: Appear counts up from zero, Change and Progress from the number
/// shown; any other moment, a one-step change (R12) and Reduced (R7) show the value at once. The
/// displayed number is what is compared (R2): a cue whose value prints the same counts nothing.
pub fn use_count_up(value: i64, cue: Cue, pace: CountPace) -> CountUp {
    let env = use_level();
    let motion = env.now();
    let own = use_motor(value);
    let tween = match pace {
        CountPace::InStep(sweep) => sweep.tween(),
        CountPace::Own(_) => Tween::from_pose(own.pose()),
    };
    let mut state = use_hook(|| {
        CopyValue::new(Counting {
            serial: 0,
            from: value,
            to: value,
            armed: tween.run(),
            last: None,
        })
    });
    let before = *state.peek();
    let mut now = before;
    if before.serial != cue.serial() || before.to != value {
        let shown = shown(before, tween);
        let from = start(cue, before.serial, shown, value, motion);
        now = Counting {
            serial: cue.serial(),
            from,
            to: value,
            armed: tween.run(),
            last: None,
        };
        if let (CountPace::Own(token), true) = (pace, from != value) {
            queue_effect(move || {
                own.play(Glide {
                    from: 0,
                    to: 1000,
                    length: token.duration(motion),
                    easing: EasingToken::Out.easing(motion),
                });
            });
        }
    }
    let number = floored(now, tween);
    now.last = Some((tween.run(), bucket(tween.elapsed()), number));
    if now != before {
        state.set(now);
    }
    CountUp { shown: number }
}

/// Where a new count starts: zero on Appear, the number shown on Change or Progress, and the
/// value itself (no count) otherwise, for a one-step change and under Reduced.
fn start(cue: Cue, last_serial: u32, shown: i64, value: i64, motion: MotionLevel) -> i64 {
    if motion == MotionLevel::Reduced {
        return value;
    }
    let fresh = last_serial != cue.serial();
    match cue.moment() {
        Moment::Appear if fresh => 0,
        Moment::Change | Moment::Progress if fresh && (value - shown).abs() > 1 => shown,
        _ => value,
    }
}

/// The number at `tween`'s frame, repainted no more often than `--t-count-step`.
fn floored(count: Counting, tween: Tween) -> i64 {
    match count.last {
        Some((run, at, number)) if run == tween.run() && at == bucket(tween.elapsed()) => number,
        _ => shown(count, tween),
    }
}

/// The number `count` shows at `tween`'s frame: its start until its move begins, then along the
/// move's curve, never past the target.
fn shown(count: Counting, tween: Tween) -> i64 {
    if tween.run() == count.armed {
        return count.from;
    }
    if tween.landed() {
        return count.to;
    }
    let span = count.to - count.from;
    let along = count.from + span * i64::from(tween.progress().0) / 1000;
    if span >= 0 {
        along.min(count.to)
    } else {
        along.max(count.to)
    }
}

/// Which `--t-count-step` window `elapsed` falls in.
fn bucket(elapsed: Duration) -> u64 {
    let step = DurationToken::CountStep
        .duration(MotionLevel::Standard)
        .as_millis()
        .max(1);
    u64::try_from(elapsed.as_millis() / step).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::bucket;
    use std::time::Duration;

    #[test]
    fn a_count_repaints_at_most_every_count_step() {
        // 0 to 93 over 700 ms is at most 22 windows of 33 ms, so at most 22 distinct numbers.
        let windows: std::collections::BTreeSet<u64> = (0..=700)
            .map(|ms| bucket(Duration::from_millis(ms)))
            .collect();
        assert_eq!(windows.len(), 22);
    }
}
