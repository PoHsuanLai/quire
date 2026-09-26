//! The widgets' soft paints (design/23-WIDGETS.md section 2, "Neumorphism & Soft UI"): the two
//! tones a soft shape is lit and shaded with (light from the top left), the raised pair and the
//! pressed pair built from them, and the ground of a night dial. Each has a light and a dark
//! value; both tones are low-contrast and the blur is wide, so a shape reads as the plate itself
//! pushed out or pressed in, never as a second material.

use super::name::VarName;
use crate::appearance::Scheme;

/// One soft paint token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoftPaint {
    /// `--soft-tone-light`: the lit tone, thrown up and to the left.
    ToneLight,
    /// `--soft-tone-dark`: the shaded tone, thrown down and to the right.
    ToneDark,
    /// `--soft-light`: a raised shape's lit side, outside it.
    Light,
    /// `--soft-dark`: a raised shape's shaded side, outside it.
    Dark,
    /// `--soft-inset-light`: a pressed shape's lit inner edge (its lower right).
    InsetLight,
    /// `--soft-inset-dark`: a pressed shape's shaded inner edge (its upper left).
    InsetDark,
    /// `--soft-night`: the ground of a night dial, darker than the plate in either scheme.
    Night,
}

impl SoftPaint {
    /// Every soft paint, in stylesheet order: the tones before the shadows that read them.
    pub const ALL: [SoftPaint; 7] = [
        SoftPaint::ToneLight,
        SoftPaint::ToneDark,
        SoftPaint::Light,
        SoftPaint::Dark,
        SoftPaint::InsetLight,
        SoftPaint::InsetDark,
        SoftPaint::Night,
    ];

    /// The custom property.
    pub fn var(self) -> VarName {
        VarName(match self {
            SoftPaint::ToneLight => "--soft-tone-light",
            SoftPaint::ToneDark => "--soft-tone-dark",
            SoftPaint::Light => "--soft-light",
            SoftPaint::Dark => "--soft-dark",
            SoftPaint::InsetLight => "--soft-inset-light",
            SoftPaint::InsetDark => "--soft-inset-dark",
            SoftPaint::Night => "--soft-night",
        })
    }

    /// The value in `scheme`.
    pub fn css(self, scheme: Scheme) -> &'static str {
        let dark = scheme == Scheme::Dark;
        match self {
            SoftPaint::ToneLight if dark => "rgba(255,255,255,.07)",
            SoftPaint::ToneLight => "rgba(255,255,255,.95)",
            SoftPaint::ToneDark if dark => "rgba(0,0,0,.62)",
            SoftPaint::ToneDark => "rgba(26,30,26,.26)",
            SoftPaint::Light => "-2px -2px 5px var(--soft-tone-light)",
            SoftPaint::Dark => "2px 2px 6px var(--soft-tone-dark)",
            SoftPaint::InsetLight => "inset -2px -2px 4px var(--soft-tone-light)",
            SoftPaint::InsetDark => "inset 2px 2px 5px var(--soft-tone-dark)",
            SoftPaint::Night if dark => "#0a0c0a",
            SoftPaint::Night => "#1a1e1a",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SoftPaint;
    use crate::appearance::Scheme;
    use std::collections::HashSet;

    #[test]
    fn every_paint_has_its_own_name_and_a_value_in_both_schemes() {
        let names: HashSet<&str> = SoftPaint::ALL.iter().map(|p| p.var().0).collect();
        assert_eq!(names.len(), SoftPaint::ALL.len());
        for paint in SoftPaint::ALL {
            for scheme in Scheme::ALL {
                assert!(!paint.css(scheme).is_empty(), "{paint:?} {scheme:?}");
            }
        }
    }
}
