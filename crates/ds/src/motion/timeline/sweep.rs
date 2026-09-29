//! A level swept from one value to another as pure arithmetic: what a frame `elapsed` into the
//! sweep draws (design/23-WIDGETS.md section 4.1, the battery ring's fill). The hook that owns
//! the clock is [`crate::motion::use_level_run::use_level_run`]; everything it decides is here, so
//! a table pins it.
//!
//! A sweep's time is the sweep itself, eased, then a tail in which what waits for the level to
//! arrive (the charging bolt) fades in, linearly.

use super::Timeline;
use crate::core::vocab::Fraction;
use crate::style::appearance::motion::MotionLevel;
use crate::style::tokens::{
    easing::{Easing, EasingToken},
    timing::DurationToken,
};
use std::time::Duration;

/// Whole, in the thousandths [`Fraction`] counts.
const WHOLE: u16 = 1000;

/// A level sweeping from one value to another, and what follows it fading in after.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sweep {
    /// Where it starts: empty on a wake, the last level drawn on a change.
    pub from: Fraction,
    /// Where it ends: the true level.
    pub to: Fraction,
    /// Whether what follows the level waits for it (a wake) or is already there (a change).
    pub tail: RunTail,
    /// How long each part takes at the motion level it plays at.
    pub timing: RunTiming,
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
}

impl Sweep {
    /// A sweep that is already at `level`: a level standing still, as under Reduced motion.
    pub fn at_rest(level: Fraction) -> Sweep {
        Sweep {
            from: level,
            to: level,
            tail: RunTail::Stays,
            timing: RunTiming {
                length: Duration::ZERO,
                easing: Easing::Linear,
                tail: Duration::ZERO,
            },
        }
    }

    /// The level `progress` (thousandths, a spring may pass 1000) of the way from `from` to `to`.
    fn level_at(&self, progress: Fraction) -> Fraction {
        if progress.0 >= WHOLE {
            return self.to;
        }
        let (from, to) = (i32::from(self.from.0), i32::from(self.to.0));
        let at = from + (to - from) * i32::from(progress.0) / i32::from(WHOLE);
        Fraction(u16::try_from(at.max(0)).unwrap_or(u16::MAX))
    }
}

impl Timeline for Sweep {
    type Frame = RunFrame;

    /// The sweep and, when what follows it waits, its tail: a tail that stays adds no time.
    fn total(&self) -> Duration {
        match self.tail {
            RunTail::Follows => self.timing.total(),
            RunTail::Stays => self.timing.length,
        }
    }

    /// The level eased from `from` to `to` across the sweep, exactly `to` from its end on; a
    /// following tail 0 across the sweep, then linear to whole, a staying one whole throughout.
    fn at(&self, elapsed: Duration) -> RunFrame {
        let timing = self.timing;
        let progress = timing.easing.at(share(elapsed, timing.length));
        let tail = match (self.tail, elapsed.checked_sub(timing.length)) {
            (RunTail::Stays, _) => Fraction(WHOLE),
            (RunTail::Follows, None) => Fraction(0),
            (RunTail::Follows, Some(after)) => share(after, timing.tail),
        };
        RunFrame {
            shown: self.level_at(progress),
            tail,
        }
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
#[cfg(test)]
mod tests {
    use super::{RunFrame, RunTail, RunTokens, Sweep};
    use crate::core::vocab::Fraction;
    use crate::motion::timeline::Timeline;
    use crate::style::appearance::motion::MotionLevel;
    use crate::style::tokens::{easing::EasingToken, timing::DurationToken};
    use std::time::Duration;

    const TOKENS: RunTokens = RunTokens {
        duration: DurationToken::Fill,
        easing: EasingToken::Out,
        tail: DurationToken::Quick,
    };

    fn sweep_of(from: u16, to: u16, tail: RunTail) -> Sweep {
        Sweep {
            from: Fraction(from),
            to: Fraction(to),
            tail,
            timing: TOKENS.timing(MotionLevel::Standard),
        }
    }

    fn at(from: u16, to: u16, ms: u64) -> RunFrame {
        sweep_of(from, to, RunTail::Follows).at(Duration::from_millis(ms))
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
            (885, 930, 566),
            (950, 930, 1000),
            (5000, 930, 1000),
        ];
        for (ms, shown, tail) in cases {
            let got = at(0, 930, ms);
            assert_eq!((got.shown.0, got.tail.0), (shown, tail), "{ms} ms: {got:?}");
        }
    }

    #[test]
    fn a_change_sweeps_down_as_well_as_up_and_keeps_its_tail() {
        let down = sweep_of(800, 300, RunTail::Stays);
        let frame = |ms| down.at(Duration::from_millis(ms));
        let standing = RunFrame {
            shown: Fraction(800),
            tail: Fraction(1000),
        };
        assert_eq!(frame(0), standing);
        let mid = frame(200).shown.0;
        assert!((301..800).contains(&mid), "{mid}");
        assert_eq!(frame(800), RunFrame::rest(Fraction(300)));
    }

    #[test]
    fn a_sweep_is_settled_only_after_its_tail() {
        let timing = TOKENS.timing(MotionLevel::Standard);
        assert_eq!(timing.total(), Duration::from_millis(950));
        let cases = [
            (RunTail::Follows, 949, false),
            (RunTail::Follows, 950, true),
            (RunTail::Stays, 799, false),
            (RunTail::Stays, 800, true),
        ];
        for (tail, ms, want) in cases {
            let got = sweep_of(0, 500, tail).settled(Duration::from_millis(ms));
            assert_eq!(got, want, "{tail:?} {ms}");
        }
    }

    #[test]
    fn a_sweep_at_rest_draws_its_level_whole_at_once() {
        let rest = Sweep::at_rest(Fraction(640));
        assert_eq!(rest.total(), Duration::ZERO);
        assert_eq!(rest.at(Duration::ZERO), RunFrame::rest(Fraction(640)));
    }

    #[test]
    fn a_sweep_is_at_its_start_inside_its_sweep_at_half_and_at_rest_at_total() {
        let sweep = sweep_of(0, 800, RunTail::Follows);
        let total = sweep.total();
        let empty = RunFrame {
            shown: Fraction(0),
            tail: Fraction(0),
        };
        assert_eq!(sweep.at(Duration::ZERO), empty);
        let half = sweep.at(total / 2);
        assert_eq!(half.tail, Fraction(0), "half of 950 ms is inside the sweep");
        assert!((1..800).contains(&half.shown.0), "{half:?}");
        assert_eq!(sweep.at(total), RunFrame::rest(Fraction(800)));
    }
}
