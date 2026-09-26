//! A level swept from one value to another as pure arithmetic: what a frame `elapsed` into the
//! sweep draws (design/23-WIDGETS.md section 4.1, the battery ring's fill). The hook that owns
//! the clock is [`crate::motion::use_level_run`]; everything it decides is here, so a table pins it.
//!
//! A sweep's time is the sweep itself, eased, then a tail in which what waits for the level to
//! arrive (the charging bolt) fades in, linearly.

use crate::appearance::MotionLevel;
use crate::components::vocab::Fraction;
use crate::tokens::{DurationToken, Easing, EasingToken};
use std::time::Duration;

/// Whole, in the thousandths [`Fraction`] counts.
const WHOLE: u16 = 1000;

/// From where to where a level runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LevelRun {
    /// Where it starts: empty on a wake, the last level drawn on a change.
    pub from: Fraction,
    /// Where it ends: the true level.
    pub to: Fraction,
    /// Whether what follows the level waits for it (a wake) or is already there (a change).
    pub tail: RunTail,
}

/// What a sweep's tail does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RunTail {
    /// Hidden across the sweep, then fades in: the sweep is an entrance.
    Follows,
    /// Whole throughout: the sweep is a change of a level already shown.
    Stays,
}

/// The tokens a sweep reads: its length and curve, and its tail's length.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RunTokens {
    /// How long the level takes to arrive.
    pub duration: DurationToken,
    /// How it decelerates.
    pub easing: EasingToken,
    /// How long what follows it takes to fade in.
    pub tail: DurationToken,
}

impl RunTokens {
    /// The tokens resolved at `level`.
    pub fn timing(self, level: MotionLevel) -> RunTiming {
        RunTiming {
            length: self.duration.duration(level),
            easing: self.easing.easing(level),
            tail: self.tail.duration(level),
        }
    }
}

/// A sweep's timing at one motion level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RunTiming {
    /// The sweep's length.
    pub length: Duration,
    /// Its curve.
    pub easing: Easing,
    /// The tail's length, after the sweep.
    pub tail: Duration,
}

impl RunTiming {
    /// The sweep and its tail.
    pub fn total(self) -> Duration {
        self.length + self.tail
    }
}

/// What one frame draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RunFrame {
    /// The level drawn this frame.
    pub shown: Fraction,
    /// How far the tail has faded in, in thousandths: 0 until the sweep ends, 1000 at rest.
    pub tail: Fraction,
}

impl RunFrame {
    /// At rest on `level`: the level drawn, the tail whole.
    pub fn rest(level: Fraction) -> Self {
        RunFrame {
            shown: level,
            tail: Fraction(WHOLE),
        }
    }

    /// The first frame of `run`.
    pub fn start(run: LevelRun) -> Self {
        RunFrame {
            shown: run.from,
            tail: match run.tail {
                RunTail::Follows => Fraction(0),
                RunTail::Stays => Fraction(WHOLE),
            },
        }
    }
}

/// Whether a sweep still has frames to draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RunPhase {
    /// The level or the tail is still moving.
    Running,
    /// Both have arrived: nothing more to draw.
    Done,
}

/// Where `run` is at `elapsed`: a tail that stays adds no time.
pub fn phase_at(run: LevelRun, timing: RunTiming, elapsed: Duration) -> RunPhase {
    let length = match run.tail {
        RunTail::Follows => timing.total(),
        RunTail::Stays => timing.length,
    };
    if elapsed >= length {
        RunPhase::Done
    } else {
        RunPhase::Running
    }
}

/// What `run` draws `elapsed` into it: the level eased from `from` to `to` across the sweep,
/// exactly `to` from its end on; a following tail 0 across the sweep, then linear to whole, a
/// staying one whole throughout.
pub fn frame_at(run: LevelRun, timing: RunTiming, elapsed: Duration) -> RunFrame {
    let progress = timing.easing.at(share(elapsed, timing.length));
    let tail = match (run.tail, elapsed.checked_sub(timing.length)) {
        (RunTail::Stays, _) => Fraction(WHOLE),
        (RunTail::Follows, None) => Fraction(0),
        (RunTail::Follows, Some(after)) => share(after, timing.tail),
    };
    RunFrame {
        shown: level_at(run, progress),
        tail,
    }
}

/// `part` of `whole` in thousandths, clamped to 0..=1000; a zero `whole` is already whole.
fn share(part: Duration, whole: Duration) -> Fraction {
    if whole.is_zero() {
        return Fraction(WHOLE);
    }
    let ratio = part.as_micros().saturating_mul(u128::from(WHOLE)) / whole.as_micros().max(1);
    Fraction(u16::try_from(ratio.min(u128::from(WHOLE))).unwrap_or(WHOLE))
}

/// The level `progress` (thousandths, a spring may pass 1000) of the way from `from` to `to`.
pub fn level_at(run: LevelRun, progress: Fraction) -> Fraction {
    if progress.0 >= WHOLE {
        return run.to;
    }
    let (from, to) = (i32::from(run.from.0), i32::from(run.to.0));
    let at = from + (to - from) * i32::from(progress.0) / i32::from(WHOLE);
    Fraction(u16::try_from(at.max(0)).unwrap_or(u16::MAX))
}

#[cfg(test)]
mod tests {
    use super::{LevelRun, RunFrame, RunPhase, RunTail, RunTokens, frame_at, phase_at};
    use crate::appearance::MotionLevel;
    use crate::components::vocab::Fraction;
    use crate::tokens::{DurationToken, EasingToken};
    use std::time::Duration;

    const TOKENS: RunTokens = RunTokens {
        duration: DurationToken::Fill,
        easing: EasingToken::Out,
        tail: DurationToken::Quick,
    };

    fn run_of(from: u16, to: u16, tail: RunTail) -> LevelRun {
        LevelRun {
            from: Fraction(from),
            to: Fraction(to),
            tail,
        }
    }

    fn at(from: u16, to: u16, ms: u64) -> RunFrame {
        let timing = TOKENS.timing(MotionLevel::Standard);
        frame_at(
            run_of(from, to, RunTail::Follows),
            timing,
            Duration::from_millis(ms),
        )
    }

    #[test]
    fn a_fill_decelerates_to_its_level_then_the_tail_fades_in() {
        // `--e-out` at Standard is (.22,.9,.3,1): .748 of the way at a quarter of 800 ms.
        let cases = [
            (0, 0, 0),
            (200, 695, 0),
            (400, 882, 0),
            (799, 930, 0),
            (800, 930, 0),
            (885, 930, 500),
            (970, 930, 1000),
            (5000, 930, 1000),
        ];
        for (ms, shown, tail) in cases {
            let got = at(0, 930, ms);
            assert_eq!((got.shown.0, got.tail.0), (shown, tail), "{ms} ms: {got:?}");
        }
    }

    #[test]
    fn a_change_sweeps_down_as_well_as_up_and_keeps_its_tail() {
        let timing = TOKENS.timing(MotionLevel::Standard);
        let down = run_of(800, 300, RunTail::Stays);
        let frame = |ms| frame_at(down, timing, Duration::from_millis(ms));
        assert_eq!(frame(0), RunFrame::start(down));
        assert_eq!(frame(0).tail, Fraction(1000));
        let mid = frame(200).shown.0;
        assert!((301..800).contains(&mid), "{mid}");
        assert_eq!(frame(800), RunFrame::rest(Fraction(300)));
    }

    #[test]
    fn a_sweep_is_done_only_after_its_tail() {
        let timing = TOKENS.timing(MotionLevel::Standard);
        assert_eq!(timing.total(), Duration::from_millis(970));
        let cases = [
            (RunTail::Follows, 969, RunPhase::Running),
            (RunTail::Follows, 970, RunPhase::Done),
            (RunTail::Stays, 799, RunPhase::Running),
            (RunTail::Stays, 800, RunPhase::Done),
        ];
        for (tail, ms, want) in cases {
            let got = phase_at(run_of(0, 500, tail), timing, Duration::from_millis(ms));
            assert_eq!(got, want, "{tail:?} {ms}");
        }
    }
}
