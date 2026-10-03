//! The double tap (ux §4.2).
//!
//! | State | Event | Next | Out |
//! |---|---|---|---|
//! | Rest | Tap at t | Armed { until: t + window } | none |
//! | Armed { until } | Tap at t, t <= until | Rest | Summon |
//! | Armed { until } | Tap at t, t > until (its Elapsed not fed yet) | Armed { until: t + window } | none |
//! | Armed { until } | Elapsed at t >= until | Rest | none |
//!
//! The machine is idle in `Rest` and runs no timer there. `Armed` keeps the window's end rather
//! than the first tap's time because [`Machine::wake`] reads the state alone.

use std::time::Duration;

use ds_core::machine::{Elapsed, Machine};
use ds_core::time::stamp::Stamp;

/// The double tap's state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tap {
    /// No tap pending.
    #[default]
    Rest,
    /// One lone tap seen; a second at or before `until` summons.
    Armed {
        /// When the window for the second tap closes (the first tap plus the window).
        until: Stamp,
    },
}

/// What moves the double tap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TapIn {
    /// A lone Command press and release ([`super::HoldOut::Tap`]).
    Tap,
    /// The window's end asked for by [`Machine::wake`] came due.
    Elapsed,
}

impl From<Elapsed> for TapIn {
    fn from(_: Elapsed) -> TapIn {
        TapIn::Elapsed
    }
}

/// What the double tap wants done.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TapOut {
    /// Summon the companion.
    Summon,
}

/// The double tap's timing (`companion.double_tap_ms`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TapParams {
    /// The longest gap between the two taps that still counts as a double.
    pub window: Duration,
}

impl Machine for Tap {
    type In = TapIn;
    type Out = TapOut;
    type Params = TapParams;

    fn step(self, input: TapIn, at: Stamp, params: &TapParams) -> (Tap, Vec<TapOut>) {
        let _ = (self, input, at, params);
        todo!("the ux 4.2 table in this module's doc")
    }

    fn wake(&self) -> Option<Stamp> {
        todo!("Armed: until; Rest: none")
    }
}
