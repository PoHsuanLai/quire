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

use crate::span;

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
    type Ctx = ();

    fn step(self, input: TapIn, at: Stamp, params: &TapParams, _: &()) -> (Tap, Vec<TapOut>) {
        match (self, input) {
            (Tap::Rest, TapIn::Tap) => (armed(at, params), Vec::new()),
            (Tap::Armed { until }, TapIn::Tap) if at <= until => (Tap::Rest, vec![TapOut::Summon]),
            (Tap::Armed { .. }, TapIn::Tap) => (armed(at, params), Vec::new()),
            (Tap::Armed { until }, TapIn::Elapsed) if at >= until => (Tap::Rest, Vec::new()),
            (state, _) => (state, Vec::new()),
        }
    }

    fn wake(&self) -> Option<Stamp> {
        match self {
            Tap::Armed { until } => Some(*until),
            Tap::Rest => None,
        }
    }
}

/// Waiting for a second tap, the window running from `at`.
fn armed(at: Stamp, params: &TapParams) -> Tap {
    Tap::Armed {
        until: span::after(at, params.window),
    }
}
