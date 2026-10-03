//! The swipe's transition (03 §4.2 and §1.5).
//!
//! | From | Input | To | Out |
//! |---|---|---|---|
//! | Idle { at } | Began | Tracking { from: at, p: at × 1000, no samples } | none |
//! | Idle { at } | Go { to }, `to != at`, `to < spaces` | Finishing { from: at, to, p0: at × 1000, v0: 0, since: now } | Committed { to } |
//! | Tracking | Changed { dx } | Tracking { p − dx × 1000 / D } (D = 1000 Magic Mouse, 400 touchpad), sample `(now, p)` | Offset { rubber(p) } |
//! | Tracking | Ended | Finishing { to: from + commit(p − from × 1000, v), p0: rubber(p), v0: v, since: now }; a target off the row is `from` | Committed { to } |
//! | Tracking | Cancelled | Finishing { to: from, p0: rubber(p), v0: 0, since: now } | Committed { to: from } |
//! | Finishing | Elapsed, `now >= since + T` | Idle { at: to } | Settled { at: to } |
//! | Finishing | Began | Tracking { from: to, p: the finish's position now, no samples } | none |
//! | Finishing | Go { m }, `m != to`, `m < spaces` | Finishing { from, to: m, p0: the position now, v0: the speed now, since: now } | Committed { to: m } |
//!
//! `v` is the least-squares slope of the samples from the last 80 ms before the lift (zero with
//! fewer than two). `T` is `finish_duration(to × 1000 − p0, v0)`; the finish's position at `now`
//! is `p0 + Δ (1 − (1 − u)³)` and its speed `3 Δ (1 − u)² / T`, with `Δ = to × 1000 − p0` and
//! `u = (now − since) / T` clamped to `0..=1`, each rounded to the nearest unit. `rubber` runs
//! over the row's ends `0` and `spaces − 1`.
//!
//! Reduced motion: `Changed` draws nothing, and where the table goes to `Finishing` the machine
//! goes straight to `Idle { at: to }` with `Committed` then `Settled`. Every other pair leaves the
//! state as it is and wants nothing (a stray change at rest, a `Go` while the fingers are down,
//! an early `Elapsed`). Only `Finishing` wakes, at `since + T`.

use ds_core::machine::Machine;
use ds_core::time::stamp::Stamp;

use super::model::{Swipe, SwipeIn, SwipeOut, SwipeParams};

impl Machine for Swipe {
    type In = SwipeIn;
    type Out = SwipeOut;
    type Params = SwipeParams;

    fn step(self, input: SwipeIn, at: Stamp, params: &SwipeParams) -> (Swipe, Vec<SwipeOut>) {
        let _ = (self, input, at, params);
        todo!("the table in this module's doc")
    }

    fn wake(&self) -> Option<Stamp> {
        todo!("Finishing: since + finish_duration(to × 1000 − p0, v0); otherwise none")
    }
}
