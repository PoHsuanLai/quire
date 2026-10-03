//! Hold Command to talk (voice §4.1).
//!
//! | State | Event | Next | Out |
//! |---|---|---|---|
//! | Rest | CommandDown at t | Down { until: t + hold } | none |
//! | Down { until } | CommandUp at t < until | Rest | Tap |
//! | Down { until } | CommandUp at t >= until (its Elapsed not fed yet) | Rest | none |
//! | Down | OtherKey, Escape or Pointer | Chorded | none (Command-C or Command-click; the mic never opened) |
//! | Down { until } | Elapsed at t >= until | Talking { until: t + max } | Start |
//! | Talking | CommandUp | Rest | End |
//! | Talking | OtherKey | Chorded | Cancel(OtherInput) |
//! | Talking | Escape | Chorded | Cancel(Escape) |
//! | Talking | Pointer | Chorded | Cancel(OtherInput) (a slow Command-click; the words are dropped) |
//! | Talking { until } | Elapsed at t >= until | Chorded | End (what was heard is sent) |
//! | Chorded | CommandUp | Rest | none |
//!
//! Every other pair leaves the state as it is and wants nothing: a repeated CommandDown (key
//! repeat), an early Elapsed, any edge but CommandDown at rest. The machine is idle in `Rest` and
//! `Chorded` with no timer. `Down` and `Talking` keep their deadline rather than their start
//! because [`Machine::wake`] reads the state alone.

use std::time::Duration;

use ds_core::machine::{Elapsed, Machine};
use ds_core::time::stamp::Stamp;

/// The Command key's hold state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HoldKey {
    /// Command is up.
    #[default]
    Rest,
    /// Command is down alone; held to `until` it starts talking.
    Down {
        /// When the hold becomes talking (the press plus the hold time).
        until: Stamp,
    },
    /// Talking: the microphone is open.
    Talking {
        /// When the longest hold ends the talk (its start plus the maximum).
        until: Stamp,
    },
    /// Command met another key or the pointer; nothing fires until it goes up.
    Chorded,
}

/// An edge the keyboard or the pointer reports while Command may be held.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyEdge {
    /// Command went down.
    CommandDown,
    /// Command went up.
    CommandUp,
    /// Any other key went down (Escape has its own edge).
    OtherKey,
    /// Escape went down.
    Escape,
    /// A pointer button went down.
    Pointer,
}

/// What moves the hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoldIn {
    /// A key or pointer edge.
    Edge(KeyEdge),
    /// The deadline asked for by [`Machine::wake`] came due.
    Elapsed,
}

impl From<Elapsed> for HoldIn {
    fn from(_: Elapsed) -> HoldIn {
        HoldIn::Elapsed
    }
}

/// Why a talk was cancelled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CancelCause {
    /// Another key or the pointer.
    OtherInput,
    /// Escape.
    Escape,
}

/// What the hold wants done.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoldOut {
    /// A lone short press: feed [`super::TapIn::Tap`] to the double tap.
    Tap,
    /// Start listening.
    Start,
    /// Stop listening and send what was heard.
    End,
    /// Stop listening and drop what was heard.
    Cancel(CancelCause),
}

/// The hold's timing (`voice.hold_ms`, `voice.max_hold_s`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HoldParams {
    /// How long Command is held alone before talking starts (300 ms).
    pub hold: Duration,
    /// The longest talk; past it the talk ends as if Command went up.
    pub max: Duration,
}

impl Machine for HoldKey {
    type In = HoldIn;
    type Out = HoldOut;
    type Params = HoldParams;

    fn step(self, input: HoldIn, at: Stamp, params: &HoldParams) -> (HoldKey, Vec<HoldOut>) {
        let _ = (self, input, at, params);
        todo!("the voice 4.1 table in this module's doc")
    }

    fn wake(&self) -> Option<Stamp> {
        todo!("Down and Talking: until; Rest and Chorded: none")
    }
}
