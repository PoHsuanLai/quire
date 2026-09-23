//! Delays and holds: the one CSS delay (`--d-fly`) and the Rust-only timer lengths
//! (design/05-MOTION.md sections 3.4 and 7.2).
//!
//! These measure intent or reading time, not motion, so they do not scale with the level
//! (design/05-MOTION.md open decision 2, proposed: hover-intent delays unchanged under Reduced).
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

use super::name::VarName;
use crate::appearance::MotionLevel;
use std::time::Duration;

/// One delay or hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DelayToken {
    /// `--d-fly` 350 ms: before a strip button's preview label shows.
    Fly,
    /// 450 ms: a hover card opens after the pointer rests.
    HoverOpen,
    /// 150 ms: a hover card closes after the pointer leaves.
    HoverClose,
    /// 400 ms: cards and labels stay warm after a close.
    HoverWarm,
    /// 5200 ms: the undo toast stays up.
    ToastHold,
    /// 18 ms per row: the heal ripple (also a CSS delay step).
    HealStep,
    /// 1600 ms: the "Sent" pill stays up.
    SentHold,
    /// 260 ms: the list re-renders after a read toggle (proposed).
    ReadReflow,
    /// 5 s: undo-send's countdown (proposed).
    SendCountdown,
    /// 1 s: one tick of that countdown (proposed).
    SendTick,
    /// 700 ms: the draft autosave debounce (proposed).
    AutosaveDebounce,
    /// 60 ms: focus the To field after the composer page mounts (proposed).
    FocusAfterMount,
    /// 1200 ms: a mentioned person chip's flash (proposed).
    FlashHold,
}

impl DelayToken {
    /// Every delay, in table order.
    pub const ALL: [DelayToken; 13] = [
        DelayToken::Fly,
        DelayToken::HoverOpen,
        DelayToken::HoverClose,
        DelayToken::HoverWarm,
        DelayToken::ToastHold,
        DelayToken::HealStep,
        DelayToken::SentHold,
        DelayToken::ReadReflow,
        DelayToken::SendCountdown,
        DelayToken::SendTick,
        DelayToken::AutosaveDebounce,
        DelayToken::FocusAfterMount,
        DelayToken::FlashHold,
    ];

    /// The custom property, for the delays the stylesheet also reads (`--d-fly`, the heal step).
    pub fn var(self) -> Option<VarName> {
        todo!()
    }

    /// How long it lasts at `level`.
    pub fn delay(self, level: MotionLevel) -> Duration {
        todo!()
    }
}
