//! Who resized a window, and whether our own request to resize it was answered. Sizes are
//! physical pixels, as winit reports them. All pure, so the rules are tested without a window;
//! `crate::window_sizer` is the handle that applies them.

use crate::window_size::Extent;
use std::time::{Duration, Instant};

/// How long after our own request a resize may still be its answer: a compositor replies on its
/// next configure, a frame or two; this is generous for a busy one and short enough that a
/// person's drag is not mistaken for it. A request not answered in this time has expired.
pub(crate) const ANSWER_WINDOW: Duration = Duration::from_millis(500);

/// Physical pixels a resize may differ from the request by, per axis: the rounding of a
/// fractional scale (logical times scale, rounded, either way).
const SLACK: u32 = 1;

/// Who resized a window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SizeOrigin {
    /// The app: the window took the size its own `request_size` asked for.
    Requested,
    /// The person (dragging an edge, tiling or zooming) or the compositor: a size the app did
    /// not ask for. Content-driven sizing stops here, so it never fights a size the person
    /// chose.
    Person,
}

/// What became of the app's last request to resize the window. It says nothing of who resized
/// the window (`SizeOrigin`): a request that expires leaves the origin as it was.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SizeRequest {
    /// Nothing is waiting: the app has not asked, or the last request was settled by a resize.
    Idle,
    /// The app asked for this size (logical px) and no resize has answered yet.
    Pending(Extent),
    /// The app asked for this size (logical px) and nothing answered in the answer window: the
    /// compositor ignored the request, or took the size it already had. The request is spent, so
    /// a resize from now on is the person's, however close it comes to this size.
    Expired(Extent),
}

/// A request, numbered, so the expiry of an old request never ends a newer one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Serial(u32);

/// Whether an expiry ended the request it was set for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Expiry {
    /// The request was still waiting: it is spent now.
    Lapsed,
    /// It was answered or replaced before: nothing changes.
    Stale,
}

/// A request we made and have not yet seen answered.
#[derive(Debug, Clone, Copy)]
struct Asked {
    serial: Serial,
    want: Extent,
    at: Instant,
}

/// Tells our own resizes from the person's. Sizes are physical pixels, as winit reports them.
#[derive(Debug, Default)]
pub(crate) struct SizeLedger {
    /// The size the window last had, so a repeat (a configure that changes nothing) is no
    /// resize.
    known: Option<Extent>,
    asked: Option<Asked>,
    issued: u32,
}

impl SizeLedger {
    /// A ledger for a window that is `known` pixels now.
    pub(crate) fn new(known: Extent) -> Self {
        SizeLedger {
            known: Some(known),
            asked: None,
            issued: 0,
        }
    }

    /// We asked for `want` at `now`. A later request replaces an earlier one; the returned
    /// serial is the one to expire.
    pub(crate) fn request(&mut self, want: Extent, now: Instant) -> Serial {
        self.issued += 1;
        let serial = Serial(self.issued);
        self.asked = Some(Asked {
            serial,
            want,
            at: now,
        });
        serial
    }

    /// The answer window of request `serial` has passed.
    pub(crate) fn expire(&mut self, serial: Serial) -> Expiry {
        match self.asked {
            Some(asked) if asked.serial == serial => {
                self.asked = None;
                Expiry::Lapsed
            }
            _ => Expiry::Stale,
        }
    }

    /// The window is `got` pixels at `now`. `None` when that is the size it already had; else
    /// who resized it: [`SizeOrigin::Requested`] when an unexpired request asked for this size
    /// (to within the rounding of the scale), [`SizeOrigin::Person`] for anything else. Either
    /// way the request is spent.
    pub(crate) fn resized(&mut self, got: Extent, now: Instant) -> Option<SizeOrigin> {
        if self.known == Some(got) {
            return None;
        }
        self.known = Some(got);
        let asked = self.asked.take();
        Some(match asked {
            Some(asked) if answers(asked, got, now) => SizeOrigin::Requested,
            _ => SizeOrigin::Person,
        })
    }
}

/// Whether `got`, at `now`, is the answer to `asked`.
fn answers(asked: Asked, got: Extent, now: Instant) -> bool {
    let within = |a: u32, b: u32| a.abs_diff(b) <= SLACK;
    now.saturating_duration_since(asked.at) <= ANSWER_WINDOW
        && within(asked.want.width, got.width)
        && within(asked.want.height, got.height)
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn e(width: u32, height: u32) -> Extent {
        Extent::new(width, height)
    }

    #[test]
    fn who_resized_the_window_follows_the_request_its_size_and_its_age() {
        let t0 = Instant::now();
        let ms = |n: u64| t0 + Duration::from_millis(n);
        // Each case: the request (size, at ms) if any, then the resize (size, at ms).
        type Case = (
            &'static str,
            Option<(Extent, u64)>,
            (Extent, u64),
            Option<SizeOrigin>,
        );
        #[rustfmt::skip]
        const CASES: &[Case] = &[
            ("no request",           None,                        (e(900, 700), 10),  Some(SizeOrigin::Person)),
            ("exact answer",         Some((e(900, 700), 0)),      (e(900, 700), 40),  Some(SizeOrigin::Requested)),
            ("rounded by one",       Some((e(900, 700), 0)),      (e(901, 699), 40),  Some(SizeOrigin::Requested)),
            ("off by two",           Some((e(900, 700), 0)),      (e(902, 700), 40),  Some(SizeOrigin::Person)),
            ("a different size",     Some((e(900, 700), 0)),      (e(1200, 800), 40), Some(SizeOrigin::Person)),
            ("at the deadline",      Some((e(900, 700), 0)),      (e(900, 700), 500), Some(SizeOrigin::Requested)),
            ("after the deadline",   Some((e(900, 700), 0)),      (e(900, 700), 501), Some(SizeOrigin::Person)),
            ("no change is no event", None,                       (e(800, 600), 10),  None),
            ("no change, requested",  Some((e(800, 600), 0)),     (e(800, 600), 10),  None),
        ];
        for (name, request, (got, at), expected) in CASES {
            let mut ledger = SizeLedger::new(e(800, 600));
            if let Some((want, at)) = request {
                ledger.request(*want, ms(*at));
            }
            assert_eq!(ledger.resized(*got, ms(*at)), *expected, "{name}");
        }
    }

    #[test]
    fn a_request_answers_one_resize_and_a_later_one_is_the_persons() {
        let t0 = Instant::now();
        let mut ledger = SizeLedger::new(e(800, 600));
        ledger.request(e(900, 700), t0);
        assert_eq!(ledger.resized(e(900, 700), t0), Some(SizeOrigin::Requested));
        assert_eq!(ledger.resized(e(950, 700), t0), Some(SizeOrigin::Person));
        assert_eq!(ledger.resized(e(900, 700), t0), Some(SizeOrigin::Person));
    }

    #[test]
    fn a_second_request_replaces_the_first() {
        let t0 = Instant::now();
        let mut ledger = SizeLedger::new(e(800, 600));
        ledger.request(e(900, 700), t0);
        ledger.request(e(1000, 750), t0);
        assert_eq!(
            ledger.resized(e(1000, 750), t0),
            Some(SizeOrigin::Requested)
        );
    }

    #[test]
    fn an_expired_request_is_spent_and_an_old_expiry_ends_nothing_newer() {
        let t0 = Instant::now();
        let mut ledger = SizeLedger::new(e(800, 600));
        let first = ledger.request(e(900, 700), t0);
        assert_eq!(ledger.expire(first), Expiry::Lapsed);
        assert_eq!(
            ledger.resized(e(900, 700), t0),
            Some(SizeOrigin::Person),
            "an expired request no longer claims the resize, however close it is"
        );

        let first = ledger.request(e(1000, 700), t0);
        let second = ledger.request(e(1100, 700), t0);
        assert_eq!(ledger.expire(first), Expiry::Stale, "replaced before");
        assert_eq!(
            ledger.resized(e(1100, 700), t0),
            Some(SizeOrigin::Requested),
            "the newer request still stands"
        );
        assert_eq!(ledger.expire(second), Expiry::Stale, "answered before");
    }
}
