//! The corner's state, timing, inputs and outputs.
//!
//! - `Armed`: the pointer entering starts the dwell.
//! - `Dwelling`: the dwell running out fires (`Fire`) and moves to `Spent` with the re-arm
//!   running. Leaving first goes back to `Armed`.
//! - `Spent`: fires nothing; it is `Armed` again once the pointer has left **and** the re-arm
//!   has run out, in either order.

use std::time::Duration;

use ds_core::time::stamp::Stamp;

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
    /// Running until this time.
    Running {
        /// When the re-arm runs out.
        until: Stamp,
    },
    /// Run out.
    Over,
}

/// The corner's state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Corner {
    /// Ready: the pointer entering starts the dwell.
    #[default]
    Armed,
    /// The pointer is in and the dwell is running.
    Dwelling {
        /// When the dwell runs out and the corner fires.
        until: Stamp,
    },
    /// Fired; waiting for the pointer to leave and the re-arm to run out.
    Spent {
        /// Where the pointer is.
        pointer: Inside,
        /// Whether the re-arm still runs.
        rearm: Rearm,
    },
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
    /// The deadline asked for by `wake` came due.
    Elapsed,
}

impl From<ds_core::machine::Elapsed> for CornerIn {
    fn from(_: ds_core::machine::Elapsed) -> CornerIn {
        CornerIn::Elapsed
    }
}

/// What to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CornerOut {
    /// The corner acts.
    Fire,
}
