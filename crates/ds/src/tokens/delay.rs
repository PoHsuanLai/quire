//! Delays and holds: the one CSS delay (`--d-fly`) and the Rust-only timer lengths
//! (design/05-MOTION.md sections 3.4 and 7.2).
//!
//! These measure intent or reading time, not motion, so they do not scale with the level
//! (design/05-MOTION.md open decision 2, proposed: hover-intent delays unchanged under Reduced).

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
    /// 120 ms: a horizontal scroll over a notification has ended once no delta has come for this
    /// long, and the swipe decides (proposed, sill Q122). Blitz forwards no scroll phase, so the
    /// end of a touchpad gesture is a quiet spell, not an event.
    SwipeQuiet,
    /// 300 ms: an `EditSurface` checks the paragraphs that changed once the typing has paused
    /// this long (proposed, design/04-COMPONENTS.md section 50).
    SpellDebounce,
}

impl DelayToken {
    /// Every delay, in table order.
    pub const ALL: [DelayToken; 15] = [
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
        DelayToken::SwipeQuiet,
        DelayToken::SpellDebounce,
    ];

    /// The custom property, for the delays the stylesheet also reads (`--d-fly`, the heal step).
    pub fn var(self) -> Option<VarName> {
        match self {
            DelayToken::Fly => Some(VarName("--d-fly")),
            DelayToken::HealStep => Some(VarName("--d-heal")),
            DelayToken::HoverOpen
            | DelayToken::HoverClose
            | DelayToken::HoverWarm
            | DelayToken::ToastHold
            | DelayToken::SentHold
            | DelayToken::ReadReflow
            | DelayToken::SendCountdown
            | DelayToken::SendTick
            | DelayToken::AutosaveDebounce
            | DelayToken::FocusAfterMount
            | DelayToken::FlashHold
            | DelayToken::SwipeQuiet
            | DelayToken::SpellDebounce => None,
        }
    }

    /// How long it lasts at `level`.
    ///
    /// The one delay that follows the level is the heal step, which is a stagger: 0 under
    /// `Reduced` (proposed, design/05-MOTION.md open decision 2), so every Reduced settle is
    /// the 94 ms section 7.1 names.
    pub fn delay(self, level: MotionLevel) -> Duration {
        Duration::from_millis(match self {
            DelayToken::Fly => 350,
            DelayToken::HoverOpen => 450,
            DelayToken::HoverClose => 150,
            DelayToken::HoverWarm => 400,
            DelayToken::ToastHold => 5200,
            DelayToken::HealStep if level == MotionLevel::Reduced => 0,
            DelayToken::HealStep => 18,
            DelayToken::SentHold => 1600,
            DelayToken::ReadReflow => 260,
            DelayToken::SendCountdown => 5000,
            DelayToken::SendTick => 1000,
            DelayToken::AutosaveDebounce => 700,
            DelayToken::FocusAfterMount => 60,
            DelayToken::FlashHold => 1200,
            DelayToken::SwipeQuiet => 120,
            DelayToken::SpellDebounce => 300,
        })
    }
}
