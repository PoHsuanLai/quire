//! The dark scheme's tone band for a re-coloured icon (design/29-SIZING.md section 11, fix F3,
//! settled 2026-09-28). Under Muted or Monochrome a colour keeps its OKLCh lightness in the light
//! scheme; in the dark one a plate kept at the dark neutral paper's lightness (L .24-.30) read
//! about 1:1 against the dark dock, and a raster's dark ink vanished on it. So in the dark:
//!
//! - a plate's stops are lifted into [`PLATE_BAND`] (.46-.56): the dark paper's base lands on .56,
//!   about 3:1 against the dock, the non-text floor; a stop already at or above .56 is kept;
//! - the art on it (a raster's pixels, the plate's ink) is remapped into [`ART_BAND`] (.62-.96),
//!   `L' = .62 + .34 L`, so no part of an icon is darker than its plate.
//!
//! One hue, tone on tone, as Monochrome exists for; the same rule for the raster and the plate.

use crate::appearance::theme::Scheme;

/// The lightness a dark plate's stops are lifted into.
pub const PLATE_BAND: (f64, f64) = (0.46, 0.56);
/// The lightness a dark icon's art is remapped into.
pub const ART_BAND: (f64, f64) = (0.62, 0.96);
/// The plate lightness that lands on the top of the band: the dark neutral paper's base.
const PLATE_KNEE: f64 = 0.30;

/// What a re-coloured colour is part of, which decides its band in the dark.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    /// A plate's gradient stop.
    Plate,
    /// A raster's pixel or a plate's ink.
    Art,
}

impl Tone {
    /// The OKLCh lightness `l` takes in `scheme`: kept in the light, banded in the dark.
    pub fn lightness(self, l: f64, scheme: Scheme) -> f64 {
        match (scheme, self) {
            (Scheme::Light, _) => l,
            (Scheme::Dark, Tone::Plate) => plate(l),
            (Scheme::Dark, Tone::Art) => art(l),
        }
    }
}

fn plate(l: f64) -> f64 {
    let (low, high) = PLATE_BAND;
    if l >= high {
        l
    } else {
        low + (high - low) * (l.max(0.0) / PLATE_KNEE).min(1.0)
    }
}

fn art(l: f64) -> f64 {
    let (low, high) = ART_BAND;
    low + (high - low) * l.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::{ART_BAND, PLATE_BAND, Tone};
    use crate::appearance::theme::Scheme;

    #[test]
    fn the_light_scheme_keeps_lightness_and_the_dark_one_bands_it() {
        #[rustfmt::skip]
        const CASES: &[(Tone, f64, f64)] = &[
            // (tone, lightness, dark lightness)
            (Tone::Plate, 0.00, 0.46),
            (Tone::Plate, 0.24, 0.54),
            (Tone::Plate, 0.30, 0.56),
            (Tone::Plate, 0.45, 0.56),
            (Tone::Plate, 0.70, 0.70),
            (Tone::Art, 0.00, 0.62),
            (Tone::Art, 0.50, 0.79),
            (Tone::Art, 1.00, 0.96),
        ];
        for (tone, l, want) in CASES {
            assert_eq!(tone.lightness(*l, Scheme::Light), *l, "{tone:?} {l} light");
            let got = tone.lightness(*l, Scheme::Dark);
            assert!(
                (got - want).abs() < 1e-9,
                "{tone:?} {l}: {got}, want {want}"
            );
        }
        assert!(PLATE_BAND.1 < ART_BAND.0, "the art sits above the plate");
    }
}
