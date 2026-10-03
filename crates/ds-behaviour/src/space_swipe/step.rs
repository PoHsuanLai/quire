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

use super::finish::{Slide, begin};
use super::model::{
    PageMilli, PagesPerSecMilli, Samples, Swipe, SwipeIn, SwipeOut, SwipeParams, SwipeSource,
};
use super::numbers::{commit, div_nearest, rubber, saturate};
use super::velocity::{lift_velocity, record};

/// What a step wants done, beside the state it leaves.
type Step = (Swipe, Vec<SwipeOut>);

impl Machine for Swipe {
    type In = SwipeIn;
    type Out = SwipeOut;
    type Params = SwipeParams;

    fn step(self, input: SwipeIn, at: Stamp, params: &SwipeParams) -> Step {
        match self {
            Swipe::Idle { at: shown } => idle(shown, input, at, params),
            Swipe::Tracking {
                from,
                source,
                p,
                samples,
            } => tracking(from, source, p, samples, input, at, params),
            Swipe::Finishing {
                from,
                to,
                p0,
                v0,
                since,
            } => finishing(from, Slide { to, p0, v0, since }, input, at, params),
        }
    }

    fn wake(&self) -> Option<Stamp> {
        match *self {
            Swipe::Finishing {
                to, p0, v0, since, ..
            } => Some(Slide { to, p0, v0, since }.ends()),
            Swipe::Idle { .. } | Swipe::Tracking { .. } => None,
        }
    }
}

/// `state`, with nothing to do.
fn unchanged(state: Swipe) -> Step {
    (state, Vec::new())
}

/// The fingers came down at `p` on a gesture that started on Space `from`.
fn track(from: u32, source: SwipeSource, p: PageMilli) -> Step {
    unchanged(Swipe::Tracking {
        from,
        source,
        p,
        samples: Samples::default(),
    })
}

/// Whether Space `to` is on the row.
fn on_row(to: u32, params: &SwipeParams) -> bool {
    to < params.spaces
}

/// The last Space on the row.
fn last_space(params: &SwipeParams) -> u32 {
    params.spaces.saturating_sub(1)
}

/// At rest on Space `shown`.
fn idle(shown: u32, input: SwipeIn, at: Stamp, params: &SwipeParams) -> Step {
    match input {
        SwipeIn::Began { source } => track(shown, source, PageMilli::of_space(shown)),
        SwipeIn::Go { to } if to != shown && on_row(to, params) => begin(
            shown,
            to,
            PageMilli::of_space(shown),
            PagesPerSecMilli(0),
            at,
            params,
        ),
        _ => unchanged(Swipe::Idle { at: shown }),
    }
}

/// The fingers are down: the row follows them and a lift or a cancel starts the finish.
fn tracking(
    from: u32,
    source: SwipeSource,
    p: PageMilli,
    samples: Samples,
    input: SwipeIn,
    at: Stamp,
    params: &SwipeParams,
) -> Step {
    let drawn = rubber(p, 0, last_space(params));
    match input {
        SwipeIn::Changed { dx } => follow(from, source, p, samples, dx, at, params),
        SwipeIn::Ended => {
            let v = lift_velocity(&samples, at);
            let to = committed_space(from, p, v, params);
            begin(from, to, drawn, v, at, params)
        }
        SwipeIn::Cancelled => begin(from, from, drawn, PagesPerSecMilli(0), at, params),
        SwipeIn::Began { .. } | SwipeIn::Go { .. } | SwipeIn::Elapsed => {
            unchanged(Swipe::Tracking {
                from,
                source,
                p,
                samples,
            })
        }
    }
}

/// The fingers moved `dx`: the row moves the other way, a page per `source`'s page of units.
fn follow(
    from: u32,
    source: SwipeSource,
    p: PageMilli,
    samples: Samples,
    dx: i32,
    at: Stamp,
    params: &SwipeParams,
) -> Step {
    let moved = div_nearest(i64::from(dx) * 1000, i64::from(source.units_per_page()));
    let p = PageMilli(saturate(i64::from(p.0) - moved));
    let state = Swipe::Tracking {
        from,
        source,
        p,
        samples: record(samples, at, p),
    };
    let draws = match params.reduced {
        super::model::Reduced::No => vec![SwipeOut::Offset {
            p: rubber(p, 0, last_space(params)),
        }],
        super::model::Reduced::Yes => Vec::new(),
    };
    (state, draws)
}

/// The Space a lift at `p` with velocity `v` commits to: one from `from` at most, and `from`
/// itself when that would leave the row.
fn committed_space(from: u32, p: PageMilli, v: PagesPerSecMilli, params: &SwipeParams) -> u32 {
    let travelled = PageMilli(saturate(
        i64::from(p.0) - i64::from(PageMilli::of_space(from).0),
    ));
    let target = i64::from(from) + i64::from(commit(travelled, v));
    u32::try_from(target)
        .ok()
        .filter(|to| on_row(*to, params))
        .unwrap_or(from)
}

/// Sliding to `slide.to`: its end settles it, the fingers catch it where it is, a `Go`
/// redirects it from where it is at the speed it has.
fn finishing(from: u32, slide: Slide, input: SwipeIn, at: Stamp, params: &SwipeParams) -> Step {
    let state = Swipe::Finishing {
        from,
        to: slide.to,
        p0: slide.p0,
        v0: slide.v0,
        since: slide.since,
    };
    match input {
        SwipeIn::Elapsed if at >= slide.ends() => (
            Swipe::Idle { at: slide.to },
            vec![SwipeOut::Settled { at: slide.to }],
        ),
        SwipeIn::Began { source } => track(slide.to, source, slide.position(at)),
        SwipeIn::Go { to } if to != slide.to && on_row(to, params) => {
            begin(from, to, slide.position(at), slide.speed(at), at, params)
        }
        _ => unchanged(state),
    }
}
