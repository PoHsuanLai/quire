//! Sizing a window to its content: the cap to the screen, and the conversions between logical
//! and physical pixels a sizer needs. All pure, so the rules are tested without a window;
//! `crate::window_sizer` is the handle that applies them.

use crate::window_size::{Extent, WindowSize};

/// The share of the screen a content-sized window may take, in percent (Preview's habit).
const CAP_PERCENT: u32 = 85;

/// How a natural size that is too big for the screen is brought down to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Fit {
    /// Each axis is capped on its own, so a long, thin natural size stays long and loses only the
    /// axis that is too big (a page that scrolls).
    PerAxis,
    /// Both axes shrink by the same factor, so the shape is kept (an image shown whole). Never
    /// larger than the natural size.
    KeepRatio,
}

impl Extent {
    /// This natural size fitted to `screen`: capped to 85% of it on each axis, and never below
    /// `least`. A `least` larger than the cap wins (a window is never smaller than its layout
    /// fits in), as does an extent of zero against any `least`. It is
    /// [`fit_with`](Extent::fit_with) [`Fit::PerAxis`].
    pub fn fit(self, screen: Extent, least: Option<Extent>) -> Extent {
        self.fit_with(screen, least, Fit::PerAxis)
    }

    /// This natural size fitted to `screen` the way `fit` says: the cap is 85% of the screen on
    /// each axis, applied per axis or to the shape as a whole, and the result is then raised
    /// per axis to `least`, which wins over the cap. A natural size with a zero axis has no
    /// shape to keep and is capped per axis.
    pub fn fit_with(self, screen: Extent, least: Option<Extent>, fit: Fit) -> Extent {
        let share = |pixels: u32| pixels.saturating_mul(CAP_PERCENT) / 100;
        let cap = Extent::new(share(screen.width), share(screen.height));
        let capped = match fit {
            Fit::PerAxis => self.per_axis_within(cap),
            Fit::KeepRatio => self.shrunk_within(cap),
        };
        let least = least.unwrap_or(Extent::new(0, 0));
        Extent::new(
            capped.width.max(least.width),
            capped.height.max(least.height),
        )
    }

    fn per_axis_within(self, cap: Extent) -> Extent {
        Extent::new(self.width.min(cap.width), self.height.min(cap.height))
    }

    /// This extent scaled down, shape kept, until it fits in `cap`; unchanged if it already does.
    fn shrunk_within(self, cap: Extent) -> Extent {
        let fits = self.width <= cap.width && self.height <= cap.height;
        if fits || self.width == 0 || self.height == 0 {
            return self.per_axis_within(cap);
        }
        let (w, h) = (u64::from(self.width), u64::from(self.height));
        let (cw, ch) = (u64::from(cap.width), u64::from(cap.height));
        // Whichever axis is further over its cap sets the factor: width when `cw / w` is the
        // smaller of the two ratios. Rounded to the nearest pixel, never past the cap.
        let scaled = |value: u64, by: u64, of: u64| ((value * by + of / 2) / of) as u32;
        match cw * h <= ch * w {
            true => Extent::new(cap.width, scaled(h, cw, w).min(cap.height)),
            false => Extent::new(scaled(w, ch, h).min(cap.width), cap.height),
        }
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
        WindowSize::fitting_with(natural, screen, least, Fit::PerAxis)
    }

    /// [`WindowSize::fitting`], choosing how a natural size that is too big shrinks
    /// ([`Fit::KeepRatio`] for content with a shape of its own).
    pub fn fitting_with(
        natural: Extent,
        screen: Option<Extent>,
        least: Extent,
        fit: Fit,
    ) -> WindowSize {
        let start = match screen {
            Some(screen) => natural.fit_with(screen, Some(least), fit),
            None => Extent::new(
                natural.width.max(least.width),
                natural.height.max(least.height),
            ),
        };
        WindowSize::new(start.width, start.height).with_least(least.width, least.height)
    }
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
    fn keeping_the_ratio_shrinks_both_axes_by_the_one_that_is_further_over() {
        // The cap of this screen is 1700 x 850.
        const SCREEN: Extent = e(2000, 1000);
        #[rustfmt::skip]
        const CASES: &[(&str, Extent, Option<Extent>, Extent)] = &[
            ("fits, unchanged",         e(800, 600),   None,              e(800, 600)),
            ("exactly the cap",         e(1700, 850),  None,              e(1700, 850)),
            ("wide, width is over",     e(3400, 1000), None,              e(1700, 500)),
            ("tall, height is over",    e(1000, 1700), None,              e(500, 850)),
            ("both over, height more",  e(4000, 4000), None,              e(850, 850)),
            ("both over, width more",   e(6800, 1700), None,              e(1700, 425)),
            ("rounds to the nearest",   e(2001, 1000), None,              e(1700, 850)),
            ("a thin natural stays thin", e(10000, 100), None,            e(1700, 17)),
            ("a zero axis is per axis", e(0, 5000),    None,              e(0, 850)),
            ("least raises per axis",   e(3400, 100),  Some(e(400, 300)), e(1700, 300)),
            ("least beats the cap",     e(5000, 4000), Some(e(1800, 900)), e(1800, 900)),
        ];
        for (name, natural, least, expected) in CASES {
            let got = natural.fit_with(SCREEN, *least, Fit::KeepRatio);
            assert_eq!(got, *expected, "{name}");
        }
    }

    #[test]
    fn keeping_the_ratio_differs_from_the_per_axis_cap_only_when_an_axis_is_over() {
        const SCREEN: Extent = e(2000, 1000);
        let natural = e(3400, 1700);
        assert_eq!(natural.fit(SCREEN, None), e(1700, 850), "per axis");
        let kept = natural.fit_with(SCREEN, None, Fit::KeepRatio);
        assert_eq!(kept, e(1700, 850), "this one is the shape of the cap");
        let wide = e(5000, 1000);
        assert_eq!(wide.fit(SCREEN, None), e(1700, 850), "per axis squashes it");
        assert_eq!(
            wide.fit_with(SCREEN, None, Fit::KeepRatio),
            e(1700, 340),
            "keep ratio shrinks it whole"
        );
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
    fn a_fitting_window_keeps_the_shape_when_asked() {
        let screen = Some(e(2000, 1000));
        let kept = WindowSize::fitting_with(e(5000, 1000), screen, e(300, 200), Fit::KeepRatio);
        assert_eq!(kept.start(), e(1700, 340));
        let unknown = WindowSize::fitting_with(e(5000, 100), None, e(300, 200), Fit::KeepRatio);
        assert_eq!(unknown.start(), e(5000, 200), "no screen, nothing to keep");
    }

    #[test]
    fn logical_and_physical_pixels_convert_by_the_scale() {
        assert_eq!(e(3840, 2160).logical_at(2.0), Some(e(1920, 1080)));
        assert_eq!(e(1000, 500).logical_at(1.5), Some(e(667, 333)));
        assert_eq!(e(1000, 500).logical_at(0.0), None);
        assert_eq!(e(1000, 500).logical_at(f64::NAN), None);
        assert_eq!(e(667, 333).physical_at(1.5), e(1001, 500));
    }
}
