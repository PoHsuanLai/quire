//! Sizing a window to its content: the cap to the screen, the screen's extent from a monitor,
//! and who resized a window (the person, or this app's own request). All pure, so the rules are
//! tested without a window; `crate::window_sizer` is the handle that applies them.

use crate::window_size::{Extent, WindowSize};
use std::time::{Duration, Instant};

/// The share of the screen a content-sized window may take, in percent (Preview's habit).
const CAP_PERCENT: u32 = 85;

/// How long after our own request a resize may still be its answer: a compositor replies on its
/// next configure, a frame or two; this is generous for a busy one and short enough that a
/// person's drag is not mistaken for it.
const ANSWER_WINDOW: Duration = Duration::from_millis(500);

/// Physical pixels a resize may differ from the request by, per axis: the rounding of a
/// fractional scale (logical times scale, rounded, either way).
const SLACK: u32 = 1;

impl Extent {
    /// This natural size fitted to `screen`: capped to 85% of it on each axis, and never below
    /// `least`. A `least` larger than the cap wins (a window is never smaller than its layout
    /// fits in), as does an extent of zero against any `least`.
    pub fn fit(self, screen: Extent, least: Option<Extent>) -> Extent {
        let cap = |natural: u32, screen: u32| natural.min(screen.saturating_mul(CAP_PERCENT) / 100);
        let least = least.unwrap_or(Extent::new(0, 0));
        Extent::new(
            cap(self.width, screen.width).max(least.width),
            cap(self.height, screen.height).max(least.height),
        )
    }

    /// `physical` pixels at `scale` (physical per logical), as logical pixels rounded to the
    /// nearest; `None` for a scale that is not a positive number.
    pub(crate) fn logical_at(self, scale: f64) -> Option<Extent> {
        let valid = scale.is_finite() && scale > 0.0;
        valid.then(|| {
            let per = |pixels: u32| (f64::from(pixels) / scale).round() as u32;
            Extent::new(per(self.width), per(self.height))
        })
    }

    /// This logical extent as physical pixels at `scale`, rounded to the nearest.
    pub(crate) fn physical_at(self, scale: f64) -> Extent {
        let per = |pixels: u32| (f64::from(pixels) * scale).round() as u32;
        Extent::new(per(self.width), per(self.height))
    }
}

impl WindowSize {
    /// A window that opens at `natural` fitted to `screen` (see [`Extent::fit`]) and keeps
    /// `least` as its least. With no `screen` known (the loop is not up, or winit lists no
    /// monitor) the window opens at `natural` raised to `least`, uncapped.
    pub fn fitting(natural: Extent, screen: Option<Extent>, least: Extent) -> WindowSize {
        let start = match screen {
            Some(screen) => natural.fit(screen, Some(least)),
            None => Extent::new(
                natural.width.max(least.width),
                natural.height.max(least.height),
            ),
        };
        WindowSize::new(start.width, start.height).with_least(least.width, least.height)
    }
}

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

/// A request we made and have not yet seen answered.
#[derive(Debug, Clone, Copy)]
struct Asked {
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
}

impl SizeLedger {
    /// A ledger for a window that is `known` pixels now.
    pub(crate) fn new(known: Extent) -> Self {
        SizeLedger {
            known: Some(known),
            asked: None,
        }
    }

    /// We asked for `want` at `now`. A later request replaces an earlier one.
    pub(crate) fn request(&mut self, want: Extent, now: Instant) {
        self.asked = Some(Asked { want, at: now });
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
    fn a_natural_size_is_capped_to_85_percent_and_never_below_the_least() {
        const SCREEN: Extent = e(2000, 1000);
        #[rustfmt::skip]
        const CASES: &[(&str, Extent, Option<Extent>, Extent)] = &[
            ("natural fits",        e(800, 600),   None,              e(800, 600)),
            ("capped on width",     e(3000, 600),  None,              e(1700, 600)),
            ("capped on both",      e(5000, 4000), None,              e(1700, 850)),
            ("exactly the cap",     e(1700, 850),  None,              e(1700, 850)),
            ("below least",         e(100, 50),    Some(e(400, 300)), e(400, 300)),
            ("least on one axis",   e(100, 600),   Some(e(400, 300)), e(400, 600)),
            ("least beats the cap", e(5000, 4000), Some(e(1800, 900)), e(1800, 900)),
            ("natural between",     e(900, 700),   Some(e(400, 300)), e(900, 700)),
        ];
        for (name, natural, least, expected) in CASES {
            assert_eq!(natural.fit(SCREEN, *least), *expected, "{name}");
        }
    }

    #[test]
    fn a_fitting_window_opens_capped_when_the_screen_is_known_and_uncapped_when_not() {
        let known = WindowSize::fitting(e(5000, 400), Some(e(2000, 1000)), e(300, 200));
        assert_eq!(known.start(), e(1700, 400));
        assert_eq!(known.least(), Some(e(300, 200)));
        let unknown = WindowSize::fitting(e(5000, 100), None, e(300, 200));
        assert_eq!(unknown.start(), e(5000, 200));
    }

    #[test]
    fn logical_and_physical_pixels_convert_by_the_scale() {
        assert_eq!(e(3840, 2160).logical_at(2.0), Some(e(1920, 1080)));
        assert_eq!(e(1000, 500).logical_at(1.5), Some(e(667, 333)));
        assert_eq!(e(1000, 500).logical_at(0.0), None);
        assert_eq!(e(1000, 500).logical_at(f64::NAN), None);
        assert_eq!(e(667, 333).physical_at(1.5), e(1001, 500));
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
}
