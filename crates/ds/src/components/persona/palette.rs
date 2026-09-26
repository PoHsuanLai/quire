//! The persona's derived palette (design/24-PERSONA.md section 3.2): every colour an OKLCh
//! value computed here, on the icon palette's eight hues (design/08 section 2.10) at a brighter
//! chroma, and five skin tones. Flat: one colour per shape, no gradient, no shading.

use super::spec::{Backdrop, HairTone, Tone};
use crate::appearance::Scheme;
use crate::space::palette::oklch_hex;

/// An OKLCh colour: lightness 0..1, chroma, hue in degrees.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Lch(pub f64, pub f64, pub f64);

impl Lch {
    /// As `#rrggbb`, fitted into sRGB.
    pub(crate) fn hex(self) -> String {
        oklch_hex(self.0, self.1, self.2)
    }

    /// Lighter by `by`, chroma scaled by `keep`.
    fn lift(self, by: f64, keep: f64) -> Lch {
        Lch((self.0 + by).clamp(0.0, 1.0), self.1 * keep, self.2)
    }
}

/// Fur and backdrop chroma: brighter than the icons' 0.07 cap, so the character carries colour.
const FUR_CHROMA: f64 = 0.12;
/// Fur lightness: light enough that ink eyes read at 28 px.
const FUR_LIGHTNESS: f64 = 0.79;

/// The hue of a fur tone; `None` for skin.
pub(crate) fn fur_hue(tone: Tone) -> Option<f64> {
    match tone {
        Tone::Peach | Tone::Sand | Tone::Honey | Tone::Umber | Tone::Cocoa => None,
        Tone::Clay => Some(40.0),
        Tone::Ochre => Some(85.0),
        Tone::Sage => Some(130.0),
        Tone::Jade => Some(175.0),
        Tone::Teal => Some(220.0),
        Tone::Slate => Some(265.0),
        Tone::Plum => Some(310.0),
        Tone::Rose => Some(355.0),
    }
}

/// The backdrop's hue.
pub(crate) fn backdrop_hue(backdrop: Backdrop) -> f64 {
    match backdrop {
        Backdrop::Clay => 40.0,
        Backdrop::Ochre => 85.0,
        Backdrop::Sage => 130.0,
        Backdrop::Jade => 175.0,
        Backdrop::Teal => 220.0,
        Backdrop::Slate => 265.0,
        Backdrop::Plum => 310.0,
        Backdrop::Rose => 355.0,
    }
}

/// Skin or fur.
pub(crate) fn tone(tone: Tone) -> Lch {
    match (tone, fur_hue(tone)) {
        (_, Some(hue)) => Lch(FUR_LIGHTNESS, FUR_CHROMA, hue),
        (Tone::Peach, None) => Lch(0.89, 0.050, 55.0),
        (Tone::Sand, None) => Lch(0.83, 0.070, 72.0),
        (Tone::Honey, None) => Lch(0.74, 0.090, 66.0),
        (Tone::Umber, None) => Lch(0.61, 0.085, 52.0),
        (Tone::Cocoa, None) => Lch(0.51, 0.070, 45.0),
        (_, None) => Lch(0.83, 0.070, 72.0),
    }
}

/// An inner ear or a muzzle: the tone, lighter and calmer.
pub(crate) fn soft(of: Tone) -> Lch {
    tone(of).lift(0.09, 0.45)
}

/// Freckles: the tone, deeper.
pub(crate) fn deep(of: Tone) -> Lch {
    tone(of).lift(-0.16, 1.1)
}

/// The blush: a warm pink a little deeper than the face, whatever the face is.
pub(crate) fn blush(of: Tone) -> Lch {
    Lch((tone(of).0 - 0.06).max(0.45), 0.12, 12.0)
}

/// The eyes, brows, mouth and nose: a warm near-black (design/08's ink, warmed).
pub(crate) fn ink() -> Lch {
    Lch(0.24, 0.015, 50.0)
}

/// A fleck of light in `Eyes::Shine`.
pub(crate) fn fleck() -> Lch {
    Lch(0.99, 0.0, 0.0)
}

/// Hair, a tuft, a sprout.
pub(crate) fn hair(hair: HairTone) -> Lch {
    match hair {
        HairTone::Ink => Lch(0.30, 0.020, 50.0),
        HairTone::Cocoa => Lch(0.42, 0.060, 50.0),
        HairTone::Auburn => Lch(0.53, 0.120, 40.0),
        HairTone::Honey => Lch(0.78, 0.110, 82.0),
        HairTone::Silver => Lch(0.87, 0.010, 250.0),
        HairTone::Rose => Lch(0.70, 0.130, 355.0),
        HairTone::Teal => Lch(0.62, 0.100, 220.0),
        HairTone::Plum => Lch(0.52, 0.110, 310.0),
    }
}

/// A bow: the backdrop's hue, saturated, so it reads against the disc it sits on.
pub(crate) fn bow(backdrop: Backdrop) -> Lch {
    Lch(0.62, 0.16, backdrop_hue(backdrop))
}

/// How far the disc's lightness keeps from the face's, so a pale face never melts into a
/// pale disc (a peach face on a clay disc did, in the combinations grid).
const GROUND_GAP: f64 = 0.08;

/// The disc behind the character: pale on light and deep on dark, stepped a shade further
/// when the face is too close to it in lightness.
pub(crate) fn ground(backdrop: Backdrop, scheme: Scheme, face: Tone) -> Lch {
    let hue = backdrop_hue(backdrop);
    let face = tone(face).0;
    match scheme {
        Scheme::Light if face > 0.91 - GROUND_GAP => Lch(0.80, 0.07, hue),
        Scheme::Light => Lch(0.91, 0.055, hue),
        Scheme::Dark if face < 0.46 + GROUND_GAP => Lch(0.34, 0.06, hue),
        Scheme::Dark => Lch(0.46, 0.06, hue),
    }
}

#[cfg(test)]
mod tests {
    use super::{Lch, ground, ink, tone};
    use crate::appearance::Scheme;
    use crate::components::persona::spec::{Backdrop, Tone};

    #[test]
    fn colours_are_hexes() {
        assert_eq!(Lch(1.0, 0.0, 0.0).hex(), "#ffffff");
        assert_eq!(ink().hex().len(), 7);
        assert_ne!(
            ground(Backdrop::Teal, Scheme::Light, Tone::Teal).hex(),
            ground(Backdrop::Teal, Scheme::Dark, Tone::Teal).hex()
        );
    }

    #[test]
    fn every_face_stands_off_the_ink() {
        // The eyes are ink on the face: every tone at least .25 lighter, so 28 px still reads.
        const TONES: [Tone; 13] = [
            Tone::Peach,
            Tone::Sand,
            Tone::Honey,
            Tone::Umber,
            Tone::Cocoa,
            Tone::Clay,
            Tone::Ochre,
            Tone::Sage,
            Tone::Jade,
            Tone::Teal,
            Tone::Slate,
            Tone::Plum,
            Tone::Rose,
        ];
        for face in TONES {
            assert!(tone(face).0 - ink().0 >= 0.25, "{face:?}");
            for scheme in Scheme::ALL {
                let disc = ground(Backdrop::Clay, scheme, face).0;
                assert!(
                    (tone(face).0 - disc).abs() >= 0.05,
                    "{face:?} on {scheme:?}"
                );
            }
        }
    }
}
