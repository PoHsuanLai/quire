//! A solid or gradient brush as a krilla paint and the opacity it paints with.

use crate::gradient;
use anyrender::{Paint, PaintRef};
use krilla::color::rgb;
use krilla::num::NormalizedF32;
use krilla::paint::Paint as KrillaPaint;
use peniko::Color;
use peniko::kurbo::Affine;

/// A brush ready for krilla: what it paints and how opaque it is overall.
pub(crate) struct Ink {
    pub(crate) paint: KrillaPaint,
    pub(crate) opacity: NormalizedF32,
}

/// `brush` as krilla ink, with `alpha` multiplied in; `None` for a brush with no vector form
/// here (an image brush is drawn separately; a renderer resource or custom brush has no PDF
/// meaning).
pub(crate) fn ink(brush: PaintRef<'_>, alpha: f32, brush_transform: Option<Affine>) -> Option<Ink> {
    match brush {
        Paint::Solid(color) => {
            let (color, own) = rgb_of(color);
            Some(Ink {
                paint: color.into(),
                opacity: opacity(own * alpha),
            })
        }
        Paint::Gradient(gradient) => Some(Ink {
            paint: gradient::paint(gradient, brush_transform.unwrap_or_default()),
            opacity: opacity(alpha),
        }),
        Paint::Image(_) | Paint::Resource(_) | Paint::Custom(_) => None,
    }
}

/// `color` as 8-bit sRGB and its alpha.
pub(crate) fn rgb_of(color: Color) -> (rgb::Color, f32) {
    let rgba = color.to_rgba8();
    (
        rgb::Color::new(rgba.r, rgba.g, rgba.b),
        f32::from(rgba.a) / 255.0,
    )
}

/// `alpha` clamped into krilla's normalized range (a NaN paints opaque).
pub(crate) fn opacity(alpha: f32) -> NormalizedF32 {
    NormalizedF32::new(alpha.clamp(0.0, 1.0)).unwrap_or(NormalizedF32::ONE)
}

#[cfg(test)]
mod tests {
    use super::{opacity, rgb_of};
    use peniko::Color;

    #[test]
    fn alpha_is_split_from_the_colour() {
        let (_, alpha) = rgb_of(Color::from_rgba8(10, 20, 30, 51));
        assert!((alpha - 0.2).abs() < 1e-6);
    }

    #[test]
    fn opacity_is_clamped() {
        const CASES: &[(f32, f32)] = &[(-1.0, 0.0), (0.5, 0.5), (2.0, 1.0), (f32::NAN, 1.0)];
        for &(given, want) in CASES {
            assert_eq!(opacity(given).get(), want, "{given}");
        }
    }
}
