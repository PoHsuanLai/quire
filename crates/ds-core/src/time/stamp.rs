//! The one "when" the design system's pure machines are given: whole milliseconds since a fixed
//! origin the caller keeps, and the clock that makes them.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// A monotonic instant in whole milliseconds since an arbitrary origin (a daemon's or a hook's
/// start). Machines take it as an argument and never read a clock; only the code that feeds a
/// machine turns a clock reading into one, with [`FrameClock`].
///
/// Milliseconds, not a `Duration`: every timer the machines ask for (hover intent, a hold, a
/// deadline) is specified in milliseconds, and a whole number compares and serialises exactly.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Stamp(pub u64);

impl Stamp {
    /// Milliseconds from `earlier` to `self`, zero if `earlier` is later.
    pub fn since(self, earlier: Stamp) -> u64 {
        self.0.saturating_sub(earlier.0)
    }

    /// `self` plus `ms` milliseconds.
    pub fn after(self, ms: u64) -> Stamp {
        Stamp(self.0.saturating_add(ms))
    }

    /// `self` plus `span`, in whole milliseconds, saturating: a configured delay as a deadline.
    pub fn after_span(self, span: Duration) -> Stamp {
        self.after(u64::try_from(span.as_millis()).unwrap_or(u64::MAX))
    }
}

/// Turns this thread's clock ([`super::clock::now`]: the wall clock, or a test harness's virtual
/// one) into [`Stamp`]s counted from an origin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameClock {
    origin: Instant,
}

impl FrameClock {
    /// A clock whose zero is `origin` (a daemon passes its start through its root context).
    pub fn new(origin: Instant) -> FrameClock {
        FrameClock { origin }
    }

    /// A clock whose zero is now, on this thread's clock (a surface with no daemon origin).
    pub fn started() -> FrameClock {
        FrameClock::new(super::clock::now())
    }

    /// `at` as a stamp; an instant before the origin is 0.
    pub fn stamp(&self, at: Instant) -> Stamp {
        Stamp(
            u64::try_from(at.saturating_duration_since(self.origin).as_millis())
                .unwrap_or(u64::MAX),
        )
    }

    /// `at` as an instant on this thread's clock: the origin plus its milliseconds. The inverse of
    /// [`Self::stamp`] up to the whole millisecond `stamp` drops.
    pub fn instant(&self, at: Stamp) -> Instant {
        self.origin + Duration::from_millis(at.0)
    }

    /// Now on this thread's clock, as a stamp.
    pub fn now(&self) -> Stamp {
        self.stamp(super::clock::now())
    }
}

#[cfg(test)]
mod tests {
    use super::{FrameClock, Stamp};
    use std::time::{Duration, Instant};

    #[test]
    fn a_stamp_and_its_instant_agree_to_the_millisecond() {
        let origin = Instant::now();
        let clock = FrameClock::new(origin);
        let cases: &[(&str, Duration, Stamp)] = &[
            ("the origin", Duration::ZERO, Stamp(0)),
            ("whole ms", Duration::from_millis(1500), Stamp(1500)),
            (
                "a part of a ms is dropped",
                Duration::from_micros(2700),
                Stamp(2),
            ),
        ];
        for (name, after, stamp) in cases {
            assert_eq!(clock.stamp(origin + *after), *stamp, "{name}: stamp");
        }
        assert_eq!(
            clock.instant(Stamp(1500)),
            origin + Duration::from_millis(1500)
        );
        assert_eq!(
            clock.stamp(origin - Duration::from_millis(1)),
            Stamp(0),
            "before the origin"
        );
    }
}
