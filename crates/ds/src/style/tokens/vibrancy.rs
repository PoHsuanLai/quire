//! The vibrancy boost baked into a flat tint (the macOS polish pass, 2026-09-24).
//!
//! macOS vibrancy saturates and brightens what shows through a material. Blitz can neither
//! saturate nor filter what is behind a surface (spike S15, S16: no `backdrop-filter`, no
//! `saturate()`), so the boost is precomputed into the tint colour instead: in OKLab, chroma is
//! raised by [`CHROMA_GAIN`] and, in the light scheme, lightness by [`LIGHT_LIFT`]. The dark
//! scheme keeps its lightness so the light ink keeps its contrast. The boost is scaled at render
//! time by the `appearance.material_vibrancy` key (percent, default 100) through the
//! `--m-vibrancy` input the root writes: 0 is the flat section 17.2 tint.

use crate::style::appearance::theme::Scheme;
use crate::style::tokens::hex::Hex;
use ds_core::colour::{oklab::Oklab, srgb::Srgb};

/// Chroma multiplier at full vibrancy: 1.4.
pub const CHROMA_GAIN: f64 = 1.4;
/// OKLab lightness added at full vibrancy in the light scheme: .012.
pub const LIGHT_LIFT: f64 = 0.012;

/// `hex` with the vibrancy boost for `scheme` baked in.
pub fn boosted(hex: Hex, scheme: Scheme) -> Hex {
    let Oklab { l, a, b } = Oklab::from(Srgb::from(hex));
    let lift = match scheme {
        Scheme::Light => LIGHT_LIFT,
        Scheme::Dark => 0.0,
    };
    let lifted = Oklab {
        l: (l + lift).min(1.0),
        a: a * CHROMA_GAIN,
        b: b * CHROMA_GAIN,
    };
    Hex::from(Srgb::from(lifted))
}

#[cfg(test)]
mod tests {
    use super::boosted;
    use crate::style::appearance::theme::Scheme;
    use crate::style::tokens::hex::Hex;
    use ds_core::colour::{oklab::Oklab, srgb::Srgb};

    #[test]
    fn oklab_round_trips_every_tint() {
        for hex in [
            Hex([248, 249, 246]),
            Hex([255, 255, 255]),
            Hex([21, 24, 20]),
            Hex([42, 47, 40]),
        ] {
            assert_eq!(Hex::from(Srgb::from(Oklab::from(Srgb::from(hex)))), hex);
        }
    }

    #[test]
    fn the_boost_lifts_the_light_tint_and_keeps_the_dark_ones_lightness() {
        let light = boosted(Hex([248, 249, 246]), Scheme::Light);
        assert!(
            light
                .0
                .iter()
                .zip([248, 249, 246])
                .all(|(got, was)| *got >= was),
            "{light:?}"
        );
        assert_ne!(light, Hex([248, 249, 246]));
        let dark = boosted(Hex([21, 24, 20]), Scheme::Dark);
        let l0 = Oklab::from(Srgb([21, 24, 20])).l;
        let l1 = Oklab::from(Srgb::from(dark)).l;
        assert!((l0 - l1).abs() < 0.005, "{l0} {l1}");
        // The dark raise's green is the more saturated for the boost.
        assert!(dark.0[1] >= 24, "{dark:?}");
    }
}
