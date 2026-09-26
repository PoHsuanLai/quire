//! Durations per motion level (design/05-MOTION.md sections 3.1-3.4).
//!
//! The four base durations and `--t-ambient` follow the level; the named durations are
//! literals that change only under `Reduced` (60 ms). Durations the prototypes wrote without a
//! name (park, nudge, C's shake, sail, boat-return, spin, the send ring, the chip flash) are named here so an
//! [`crate::Anim`] can point at them.
//!
//! `Reduced` is 60 ms for every token of [`DurationKind::Motion`], the named ones included
//! (design/05-MOTION.md section 3.2, "named durations (3.4) ... 60ms each"); a
//! [`DurationKind::Hold`] token keeps its Standard value instead (wave 1 amendment: a held
//! state, not something that moves, so shortening it to 60 ms would make it unreadable rather
//! than calmer). `--t-big-heavy` is `--t-big` x 1.15 at each level.

use super::name::VarName;
use crate::appearance::MotionLevel;
use std::time::Duration;

/// Whether Reduced motion shortens a [`DurationToken`] to 60 ms, or the token times a held
/// state a person must still be able to register regardless of motion level. Data on the
/// token (`DurationToken::kind`) rather than a special case in its duration table, so a new
/// hold is one match arm, not a second code path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DurationKind {
    /// Shortened to 60 ms under Reduced, like every other transition.
    Motion,
    /// Keeps its Standard value under Reduced.
    Hold,
}

/// One duration token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DurationToken {
    /// `--t-tap` 90 ms: press feedback.
    Tap,
    /// `--t-quick` 170 ms: colour and opacity.
    Quick,
    /// `--t-move` 250 ms: small movement.
    Move,
    /// `--t-big` 420 ms: entrances and exits (Calm 300, Extra 560).
    Big,
    /// `--t-ambient` 5 s: the breathing halo.
    Ambient,
    /// `--t-big-heavy` = `--t-big` x 1.15: an unread row's exit.
    BigHeavy,
    /// `--t-spark` 520 ms: star sparks.
    Spark,
    /// `--t-curl` 560 ms: the snooze exit.
    Curl,
    /// `--t-curl-heavy` = 560 x 1.15 = 644 ms: an unread snooze (proposed, 05-MOTION open decision 3).
    CurlHeavy,
    /// `--t-crumple-heavy` = `--t-big` x 1.15: an unread row's trash (freeze amendment, wave 1).
    CrumpleHeavy,
    /// `--t-send` 620 ms: compose-send.
    Send,
    /// `--t-float` 900 ms: the zZ floater, the destination pulse period.
    Float,
    /// `--t-hc-out` 120 ms: the hover card leaving.
    HcOut,
    /// `--t-scene` 380 ms: the Space layer cross-fade.
    Scene,
    /// `--t-shake` 420 ms: `shake-x`.
    Shake,
    /// `--t-park` 420 ms: the composer page parking.
    Park,
    /// `--t-nudge` 520 ms: outbox retry.
    Nudge,
    /// `--t-shake-long` 560 ms: C's outbox `shake`.
    ShakeLong,
    /// `--t-sail` 1150 ms: the orphaned boat.
    Sail,
    /// `--t-boat-return` 900 ms: the orphaned boat's return.
    BoatReturn,
    /// `--t-spin` 1100 ms: the busy halo, linear.
    Spin,
    /// `--t-send-ring` 5 s: the undo-send countdown ring, linear. [`DurationKind::Hold`]: the
    /// ring shows how long a send can still be taken back, which Reduced must not shorten.
    SendRing,
    /// `--t-flash` 1200 ms: a mentioned person chip's ring, held (design/04-COMPONENTS.md
    /// section 10, design/06-INTERACTIONS.md section 2.5, `S:2119`; proposed).
    /// [`DurationKind::Hold`]: keeps 1200 ms under Reduced instead of shortening to 60 ms, so
    /// the ring is still visible (FINDINGS.md "W1 integration" left this open; resolved here).
    Flash,
    /// `--t-awake` 20 s: how long a persona stays awake (blinking and breathing) after a wake
    /// or a mood change, and the length of its one breathing run (design/24-PERSONA.md).
    Awake,
    /// `--t-drift` 2400 ms: a sleeping persona's single `z` rising and fading (design/24).
    Drift,
    /// `--t-fill` 800 ms: a battery ring sweeping from empty (or its last level) to its level,
    /// its percentage counting alongside (design/23-WIDGETS.md sections 1.1 and 4.1). Driven
    /// frame by frame from Rust at `--e-out`; no keyframe plays it.
    Fill,
}

impl DurationToken {
    /// Every duration token, in stylesheet order.
    pub const ALL: [DurationToken; 26] = [
        DurationToken::Tap,
        DurationToken::Quick,
        DurationToken::Move,
        DurationToken::Big,
        DurationToken::Ambient,
        DurationToken::BigHeavy,
        DurationToken::Spark,
        DurationToken::Curl,
        DurationToken::CurlHeavy,
        DurationToken::CrumpleHeavy,
        DurationToken::Send,
        DurationToken::Float,
        DurationToken::HcOut,
        DurationToken::Scene,
        DurationToken::Shake,
        DurationToken::Park,
        DurationToken::Nudge,
        DurationToken::ShakeLong,
        DurationToken::Sail,
        DurationToken::BoatReturn,
        DurationToken::Spin,
        DurationToken::SendRing,
        DurationToken::Flash,
        DurationToken::Awake,
        DurationToken::Drift,
        DurationToken::Fill,
    ];

    /// The custom property: `--t-tap`, `--t-big-heavy`, …
    pub fn var(self) -> VarName {
        VarName(match self {
            DurationToken::Tap => "--t-tap",
            DurationToken::Quick => "--t-quick",
            DurationToken::Move => "--t-move",
            DurationToken::Big => "--t-big",
            DurationToken::Ambient => "--t-ambient",
            DurationToken::BigHeavy => "--t-big-heavy",
            DurationToken::Spark => "--t-spark",
            DurationToken::Curl => "--t-curl",
            DurationToken::CurlHeavy => "--t-curl-heavy",
            DurationToken::CrumpleHeavy => "--t-crumple-heavy",
            DurationToken::Send => "--t-send",
            DurationToken::Float => "--t-float",
            DurationToken::HcOut => "--t-hc-out",
            DurationToken::Scene => "--t-scene",
            DurationToken::Shake => "--t-shake",
            DurationToken::Park => "--t-park",
            DurationToken::Nudge => "--t-nudge",
            DurationToken::ShakeLong => "--t-shake-long",
            DurationToken::Sail => "--t-sail",
            DurationToken::BoatReturn => "--t-boat-return",
            DurationToken::Spin => "--t-spin",
            DurationToken::SendRing => "--t-send-ring",
            DurationToken::Flash => "--t-flash",
            DurationToken::Awake => "--t-awake",
            DurationToken::Drift => "--t-drift",
            DurationToken::Fill => "--t-fill",
        })
    }

    /// How long it lasts at `level`.
    pub fn duration(self, level: MotionLevel) -> Duration {
        Duration::from_millis(self.millis(level))
    }

    /// Whether Reduced shortens this token to 60 ms like every other transition, or the token
    /// is a held state a person must still be able to register and so keeps its Standard value
    /// (wave 1 amendment, FINDINGS.md "W1 integration": "whether holds should be exempt is an
    /// open design question" — resolved here by making it data on the token rather than a
    /// special case in [`Self::millis`]).
    pub fn kind(self) -> DurationKind {
        match self {
            // The undo window is time a person has to act, not motion (mailo gaps 3).
            DurationToken::Flash | DurationToken::SendRing => DurationKind::Hold,
            _ => DurationKind::Motion,
        }
    }

    /// The table, in milliseconds.
    fn millis(self, level: MotionLevel) -> u64 {
        const REDUCED: u64 = 60;
        if level == MotionLevel::Reduced && self.kind() == DurationKind::Motion {
            return REDUCED;
        }
        match (self, level) {
            (DurationToken::Big, MotionLevel::Calm) => 300,
            (DurationToken::Big, MotionLevel::Extra) => 560,
            // `calc(var(--t-big) * 1.15)`, rounded as section 3.4 lists it: Calm 345,
            // Standard 483, Extra 644. Crumple runs at `--t-big`, so its heavy form is the same.
            (DurationToken::BigHeavy | DurationToken::CrumpleHeavy, level) => {
                (DurationToken::Big.millis(level) * 115).div_ceil(100)
            }
            (DurationToken::Tap, _) => 90,
            (DurationToken::Quick, _) => 170,
            (DurationToken::Move, _) => 250,
            (DurationToken::Big, _) => 420,
            (DurationToken::Ambient, _) => 5000,
            (DurationToken::Spark, _) => 520,
            (DurationToken::Curl, _) => 560,
            // Proposed: 560 x 1.15 (design/05-MOTION.md open decision 3).
            (DurationToken::CurlHeavy, _) => 644,
            (DurationToken::Send, _) => 620,
            (DurationToken::Float, _) => 900,
            (DurationToken::HcOut, _) => 120,
            (DurationToken::Scene, _) => 380,
            (DurationToken::Shake, _) => 420,
            (DurationToken::Park, _) => 420,
            (DurationToken::Nudge, _) => 520,
            (DurationToken::ShakeLong, _) => 560,
            (DurationToken::Sail, _) => 1150,
            (DurationToken::BoatReturn, _) => 900,
            (DurationToken::Spin, _) => 1100,
            (DurationToken::SendRing, _) => 5000,
            (DurationToken::Flash, _) => 1200,
            (DurationToken::Awake, _) => 20000,
            (DurationToken::Drift, _) => 2400,
            (DurationToken::Fill, _) => 800,
        }
    }
}
