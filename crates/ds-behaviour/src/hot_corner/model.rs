//! The corner's phase, timers, inputs and outputs.
//!
//! - `Armed`: the pointer entering starts the dwell (`StartDwell`, a fresh token).
//! - `Dwelling`: the dwell running out fires (`Fire`) and moves to `Spent` with the re-arm
//!   running (`StartRearm`). Leaving first goes back to `Armed` (the dwell's timer is stale by
//!   its token).
//! - `Spent`: fires nothing; it is `Armed` again once the pointer has left **and** the re-arm
//!   has run out, in either order.

use std::time::Duration;

/// A timer's identity; an elapsed timer with another token is stale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Token(pub u32);

/// Where the pointer is, for `Spent`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Inside {
    /// In the corner.
    In,
    /// Out of it.
    Out,
}

/// Whether `Spent`'s re-arm is still running.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rearm {
    /// Running; its elapse comes back with this token.
    Running(Token),
    /// Run out.
    Over,
}

/// The corner's phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Phase {
    /// Ready: the pointer entering starts the dwell.
    #[default]
    Armed,
    /// The pointer is in and the dwell with this token is running.
    Dwelling(Token),
    /// Fired; waiting for the pointer to leave and the re-arm to run out.
    Spent {
        /// Where the pointer is.
        pointer: Inside,
        /// Whether the re-arm still runs.
        rearm: Rearm,
    },
}

/// The machine: its phase and the next token to hand out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CornerMachine {
    /// The phase.
    pub phase: Phase,
    /// The token the next timer gets.
    pub next: Token,
}

/// The corner's timing (`hot_corners.dwell_ms`, `hot_corners.rearm_ms`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CornerParams {
    /// How long the pointer rests in the corner before it fires.
    pub dwell: Duration,
    /// After firing, how long before the corner may fire again (once the pointer has left).
    pub rearm: Duration,
}

/// What happened.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CornerIn {
    /// The pointer entered the corner (with the corner's modifiers held, where it has any).
    Enter,
    /// The pointer left the corner.
    Leave,
    /// The dwell timer with this token ran out.
    DwellElapsed(Token),
    /// The re-arm timer with this token ran out.
    RearmElapsed(Token),
}

/// What to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CornerOut {
    /// Wait this long, then feed `DwellElapsed(token)`.
    StartDwell(Token, Duration),
    /// Wait this long, then feed `RearmElapsed(token)`.
    StartRearm(Token, Duration),
    /// The corner acts.
    Fire,
}
