//! The range an eased scroll listener's offset may take, and the two things it does at the ends:
//! clamp to them, or stretch past them like the rubber band a touchpad flick shows. Pure.
//!
//! The stretch is the engine's own band (`blitz_kit::scroll::rubber::Band`, design/11 §11.3.7:
//! `(1 - 1 / (over * 0.55 / d + 1)) * d` for `over` px asked past an edge on a viewport of `d`
//! px), so a listener's overscroll matches what the document's own scrollers do.

use blitz_kit::scroll::rubber::{AtEdge, Band};

/// The lowest and highest offset a scroller may rest at, in px.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollBounds {
    pub min: f64,
    pub max: f64,
}

impl ScrollBounds {
    /// The range `min..=max`; given the ends the wrong way round, they are swapped.
    pub fn new(min: f64, max: f64) -> ScrollBounds {
        if min <= max {
            ScrollBounds { min, max }
        } else {
            ScrollBounds { min: max, max: min }
        }
    }

    /// The range of a scroller whose content is `content` px long in a viewport of `viewport` px:
    /// `0` to the overflow, and just `0` when the content fits.
    pub fn of_content(content: f64, viewport: f64) -> ScrollBounds {
        ScrollBounds::new(0.0, (content - viewport).max(0.0))
    }

    /// `offset` held inside the range.
    pub fn clamp(self, offset: f64) -> f64 {
        offset.clamp(self.min, self.max)
    }

    /// How far `offset` is past the range: negative before the start, positive after the end, zero
    /// inside it.
    pub fn overshoot(self, offset: f64) -> f64 {
        if offset < self.min {
            offset - self.min
        } else if offset > self.max {
            offset - self.max
        } else {
            0.0
        }
    }

    /// The offset to show for a raw `offset`: unchanged inside the range, and past an end the end
    /// plus `band`'s stretch of the overshoot on a viewport of `viewport` px, in the direction of
    /// the overshoot. The stretch never reaches `viewport`.
    pub fn stretch(self, band: &Band, offset: f64, viewport: f64) -> f64 {
        let over = self.overshoot(offset);
        if over == 0.0 {
            return offset;
        }
        let shown = band.stretch(over.abs(), viewport, AtEdge::Inside);
        let edge = self.clamp(offset);
        edge + shown.copysign(over)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blitz_kit::scroll::config::ScrollSettings;

    fn band() -> Band {
        Band::from_settings(&ScrollSettings::default())
    }

    #[test]
    fn an_offset_is_clamped_to_the_range() {
        let bounds = ScrollBounds::new(0.0, 500.0);
        // name, offset, clamped
        let cases = [
            ("inside", 120.0, 120.0),
            ("at the start", 0.0, 0.0),
            ("at the end", 500.0, 500.0),
            ("before", -80.0, 0.0),
            ("after", 900.0, 500.0),
        ];
        for (name, offset, want) in cases {
            assert_eq!(bounds.clamp(offset), want, "{name}");
        }
    }

    #[test]
    fn the_range_comes_from_the_content_and_the_viewport() {
        // name, content, viewport, range
        let cases = [
            ("overflow", 1500.0, 1000.0, (0.0, 500.0)),
            ("fits", 600.0, 1000.0, (0.0, 0.0)),
            ("exact", 1000.0, 1000.0, (0.0, 0.0)),
        ];
        for (name, content, viewport, (min, max)) in cases {
            assert_eq!(
                ScrollBounds::of_content(content, viewport),
                ScrollBounds { min, max },
                "{name}"
            );
        }
        assert_eq!(
            ScrollBounds::new(10.0, -10.0),
            ScrollBounds {
                min: -10.0,
                max: 10.0
            }
        );
    }

    #[test]
    fn the_stretch_is_the_rubber_bands_past_an_end_and_nothing_inside() {
        let bounds = ScrollBounds::new(0.0, 500.0);
        let d = 1000.0;
        let band_of = |over: f64| (1.0 - 1.0 / (over * 0.55 / d + 1.0)) * d;
        // name, offset, shown
        let cases = [
            ("inside", 250.0, 250.0),
            ("at the end", 500.0, 500.0),
            ("100 past the end", 600.0, 500.0 + band_of(100.0)),
            ("1000 past the end", 1500.0, 500.0 + band_of(1000.0)),
            ("100 before the start", -100.0, -band_of(100.0)),
        ];
        for (name, offset, want) in cases {
            let got = bounds.stretch(&band(), offset, d);
            assert!((got - want).abs() < 1e-9, "{name}: {got} != {want}");
        }
    }

    #[test]
    fn the_stretch_stays_under_the_viewport_however_far_it_is_pulled() {
        let bounds = ScrollBounds::new(0.0, 0.0);
        let shown = bounds.stretch(&band(), 1.0e9, 800.0);
        assert!(shown > 0.0 && shown < 800.0, "{shown}");
    }
}
