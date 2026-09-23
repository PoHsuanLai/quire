//! Durations per motion level (design/05-MOTION.md sections 3.1-3.4).
//!
//! The four base durations and `--t-ambient` follow the level; the named durations are
//! literals that change only under `Reduced` (60 ms). Durations the prototypes wrote without a
//! name (park, nudge, C's shake, sail, boat-return, spin, the send ring) are named here so an
//! [`crate::Anim`] can point at them.
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

use super::name::VarName;
use crate::appearance::MotionLevel;
use std::time::Duration;

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
    /// `--t-send-ring` 5 s: the undo-send countdown ring, linear.
    SendRing,
}

impl DurationToken {
    /// Every duration token, in stylesheet order.
    pub const ALL: [DurationToken; 21] = [
        DurationToken::Tap,
        DurationToken::Quick,
        DurationToken::Move,
        DurationToken::Big,
        DurationToken::Ambient,
        DurationToken::BigHeavy,
        DurationToken::Spark,
        DurationToken::Curl,
        DurationToken::CurlHeavy,
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
    ];

    /// The custom property: `--t-tap`, `--t-big-heavy`, …
    pub fn var(self) -> VarName {
        todo!()
    }

    /// How long it lasts at `level`.
    pub fn duration(self, level: MotionLevel) -> Duration {
        todo!()
    }
}
