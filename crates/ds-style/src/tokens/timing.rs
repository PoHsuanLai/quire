//! Durations per motion level (design/30-CATALOGUE.md section 1.2).
//!
//! The durations are literals that change only under `Reduced`, where every moving one is
//! `--t-quick`: what still moves is a cross-fade (section 1.1). A [`DurationKind::Hold`] token
//! keeps its Standard value instead (a held state, or the spinner's step, is not something that
//! moves, so shortening it would make it unreadable rather than calmer).

use crate::appearance::motion::MotionLevel;
use crate::tokens::token::{CssValue, Token, TokenScope};
use ds_core::word::Word;
use std::time::Duration;

/// Whether Reduced motion shortens a [`DurationToken`] to `--t-quick`, or the token times a held
/// state a person must still be able to register regardless of motion level. Data on the
/// token (`DurationToken::kind`) rather than a special case in its duration table, so a new
/// hold is one match arm, not a second code path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DurationKind {
    /// Shortened to `--t-quick` under Reduced, like every other transition.
    Motion,
    /// Keeps its Standard value under Reduced.
    Hold,
}

/// One duration token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "t-", kind = fixed, css = duration_css)]
#[non_exhaustive]
pub enum DurationToken {
    /// `--t-quick` 120 ms: colour and opacity, closes, cross-fades, and every moving duration
    /// under Reduced.
    Quick,
    /// `--t-move` 200 ms: small movement.
    Move,
    /// `--t-big` 280 ms: entrances and exits, and the Space colour cross-fade.
    Big,
    /// `--t-spin-step` 83 ms: one of the spinner's twelve spokes, a turn a second.
    /// [`DurationKind::Hold`]: the spinner keeps turning under Reduced.
    SpinStep,
    /// `--t-turn` 1000 ms: one turn of a busy button's icon (`turn`), linear.
    /// Shortened to `--t-quick` under Reduced like every motion, where `turn` plays its still form.
    Turn,
    /// `--t-shake` 420 ms: `shake-x`.
    Shake,
    /// `--t-send-ring` 5 s: the undo-send countdown ring, linear. [`DurationKind::Hold`]: the
    /// ring shows how long a send can still be taken back, which Reduced must not shorten.
    SendRing,
    /// `--t-awake` 20 s: how long an animated emoji stays awake (playing its loop) after a wake
    /// or a mood change (design/25-EMOJI.md section 5).
    Awake,
    /// `--t-idle-dim` 2000 ms: the pre-screen-off idle overlay's fade in to its own level
    /// (`idle.dim_level_pct`), Rust-driven like a `Sweep` rather than a keyframe, because waking
    /// must snap mid-fade and a CSS animation cannot retarget that way without a restyle
    /// (design/22-SETTINGS.md section 3.24 `idle.dim_s`/`idle.dim_level_pct`;
    /// `ds::motion::detail::idle_dim`). Reduced: 60 ms like every other transition by this table, but the
    /// primitive itself never plays it that long — Reduced jumps straight to the level, the way
    /// `Sweep`'s own `Stand` plan does (R7, design/26 section 3.3).
    IdleDim,
    /// `--t-orb-listen` 12 s (proposed): one turn of the companion orb while it listens, the
    /// slowest. [`DurationKind::Hold`]: the orb is inactive under Reduced, so the period is never
    /// shortened to a cross-fade (design/32 section 3).
    OrbListen,
    /// `--t-orb-work` 8 s (proposed): one turn while it works out of sight.
    /// [`DurationKind::Hold`], as `--t-orb-listen`.
    OrbWork,
    /// `--t-orb-act` 5 s (proposed): one turn while it acts in a window, the quickest.
    /// [`DurationKind::Hold`], as `--t-orb-listen`.
    OrbAct,
}

impl DurationToken {
    /// How long it lasts at `level`.
    pub fn duration(self, level: MotionLevel) -> Duration {
        Duration::from_millis(self.millis(level))
    }

    /// Whether Reduced shortens this token to 60 ms like every other transition, or the token
    /// is a held state a person must still be able to register and so keeps its Standard value
    /// (data on the token rather than a special case in [`Self::millis`]).
    pub fn kind(self) -> DurationKind {
        match self {
            // The undo window is time a person has to act, not motion.
            DurationToken::SendRing => DurationKind::Hold,
            // The spinner keeps turning under Reduced, as macOS's does.
            DurationToken::SpinStep => DurationKind::Hold,
            // The orb's period is a speed, not a transition; Reduced makes the orb inactive
            // instead of making it turn at a cross-fade's pace.
            DurationToken::OrbListen | DurationToken::OrbWork | DurationToken::OrbAct => {
                DurationKind::Hold
            }
            _ => DurationKind::Motion,
        }
    }

    /// The table, in milliseconds.
    fn millis(self, level: MotionLevel) -> u64 {
        if level == MotionLevel::Reduced && self.kind() == DurationKind::Motion {
            return DurationToken::Quick.millis(MotionLevel::Standard);
        }
        match (self, level) {
            (DurationToken::Quick, _) => 120,
            (DurationToken::Move, _) => 200,
            (DurationToken::Big, _) => 280,
            (DurationToken::SpinStep, _) => 83,
            (DurationToken::Turn, _) => 1000,
            (DurationToken::Shake, _) => 420,
            (DurationToken::SendRing, _) => 5000,
            (DurationToken::Awake, _) => 20000,
            (DurationToken::IdleDim, _) => 2000,
            (DurationToken::OrbListen, _) => 12000,
            (DurationToken::OrbWork, _) => 8000,
            (DurationToken::OrbAct, _) => 5000,
        }
    }
}

/// A duration as the stylesheet writes it: whole milliseconds at the scope's motion level.
fn duration_css(token: DurationToken, scope: TokenScope) -> CssValue {
    CssValue::computed(format!("{}ms", token.duration(scope.motion).as_millis()))
}
