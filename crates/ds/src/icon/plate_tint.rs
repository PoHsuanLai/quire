//! A plate re-coloured for the icon style (sill FINDINGS Q72; design/08-ICONS.md section 4).
//!
//! Under Muted or Monochrome a third-party icon's raster goes through [`super::retint`], but its
//! plate is drawn by the stylesheet from the family's stops: in dark mode a Monochrome dock
//! showed a tinted glyph on the untinted `#2A2E28` paper. A consumer cannot restyle `.ds-plate`
//! (`Rule::DsInternals`) or write the colours inline (`Rule::HexColour`), so the plate takes
//! the style itself: its two stops and its ink go through the same OKLCh rule as the raster
//! ([`super::retint::recolour`], one implementation), for both schemes, and are handed to the
//! stylesheet as custom properties on the plate (`--plate-base-l` ... `--plate-ink-d`), which the
//! sheet reads under the root's `data-theme`, as a Space dot's `--dot-c*` are (mailo gaps 3).

use super::family::{NEUTRAL_DARK, PlateFamily};
use super::retint::{IconStyle, Tint, recolour};
use crate::appearance::Scheme;
use crate::tokens::{Hex, VarName};

/// How a plate is re-coloured: `icons.style` when it is not Colour.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlateTint {
    /// Chroma scaled by [`super::retint::MUTED_SCALE`]; hue and lightness kept.
    Muted,
    /// The tint's hue and chroma, eased near black and white; lightness kept.
    Monochrome(Tint),
}

impl PlateTint {
    /// The plate's re-colouring for `style` and `tint`, the pair sill already hands `retint`:
    /// `None` for Colour, which draws the family as designed.
    pub fn of(style: IconStyle, tint: Tint) -> Option<PlateTint> {
        match style {
            IconStyle::Colour => None,
            IconStyle::Muted => Some(PlateTint::Muted),
            IconStyle::Monochrome => Some(PlateTint::Monochrome(tint)),
        }
    }

    /// The `data-icon-style` word the stylesheet selects the tinted stops by.
    pub fn slug(self) -> &'static str {
        match self {
            PlateTint::Muted => "muted",
            PlateTint::Monochrome(_) => "monochrome",
        }
    }

    /// One opaque colour re-coloured as `retint` re-colours an opaque pixel.
    fn recolour(self, colour: Hex) -> Hex {
        let (style, tint) = match self {
            PlateTint::Muted => (IconStyle::Muted, Tint::NEUTRAL),
            PlateTint::Monochrome(tint) => (IconStyle::Monochrome, tint),
        };
        Hex(recolour(colour.0, u8::MAX, style, tint))
    }
}

/// A plate's paint in one scheme: the 135 degree gradient's two stops and the glyph colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlateStops {
    /// The light stop, top left.
    pub base: Hex,
    /// The deep stop, bottom right.
    pub deep: Hex,
    /// The glyph (and symbolic icon) colour.
    pub ink: Hex,
}

impl PlateStops {
    /// The family as the stylesheet draws it in `scheme`: only the neutral plate changes in the
    /// dark (design/08 section 4.1).
    pub fn of(family: PlateFamily, scheme: Scheme) -> PlateStops {
        match (family, scheme) {
            (PlateFamily::Neutral, Scheme::Dark) => {
                let (base, deep, ink) = NEUTRAL_DARK;
                PlateStops { base, deep, ink }
            }
            _ => {
                let (base, deep) = family.stops();
                PlateStops {
                    base,
                    deep,
                    ink: family.glyph(),
                }
            }
        }
    }

    /// These stops and ink re-coloured by `tint`.
    pub fn tinted(self, tint: PlateTint) -> PlateStops {
        PlateStops {
            base: tint.recolour(self.base),
            deep: tint.recolour(self.deep),
            ink: tint.recolour(self.ink),
        }
    }
}

/// The custom properties a tinted plate writes, light scheme then dark, each base, deep, ink.
pub(crate) const PLATE_TINT_VARS: [[VarName; 3]; 2] = [
    [
        VarName("--plate-base-l"),
        VarName("--plate-deep-l"),
        VarName("--plate-ink-l"),
    ],
    [
        VarName("--plate-base-d"),
        VarName("--plate-deep-d"),
        VarName("--plate-ink-d"),
    ],
];

/// The inline declarations of `family` tinted by `tint`, both schemes:
/// `--plate-base-l:#…;…;--plate-ink-d:#…;`.
pub(crate) fn tint_style(family: PlateFamily, tint: PlateTint) -> String {
    Scheme::ALL
        .into_iter()
        .zip(PLATE_TINT_VARS)
        .flat_map(|(scheme, names)| {
            let stops = PlateStops::of(family, scheme).tinted(tint);
            names.into_iter().zip([stops.base, stops.deep, stops.ink])
        })
        .map(|(name, colour)| format!("{}:{};", name.as_str(), colour.css()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{PlateStops, PlateTint, tint_style};
    use crate::appearance::Scheme;
    use crate::icon::family::PlateFamily;
    use crate::icon::retint::{IconStyle, Tint};
    use crate::space::PRESETS;
    use crate::tokens::Hex;

    fn hue_and_chroma(colour: Hex) -> Tint {
        Tint::from_hex(&colour.css()).expect("a hex colour")
    }

    fn hue_gap(a: f32, b: f32) -> f32 {
        ((a - b + 540.0).rem_euclid(360.0) - 180.0).abs()
    }

    #[test]
    fn colour_is_no_tint_and_the_other_styles_are() {
        let tint = Tint::space(PRESETS[0].dots);
        assert_eq!(PlateTint::of(IconStyle::Colour, tint), None);
        assert_eq!(
            PlateTint::of(IconStyle::Muted, tint),
            Some(PlateTint::Muted)
        );
        assert_eq!(
            PlateTint::of(IconStyle::Monochrome, tint),
            Some(PlateTint::Monochrome(tint))
        );
    }

    /// The dark neutral plate, the case Q72 reported, takes the Work and the Home tint's hue in
    /// both stops and its ink; the light paper keeps its white stop white.
    #[test]
    fn a_monochrome_neutral_plate_takes_the_tints_hue() {
        for preset in [0, 1] {
            let tint = Tint::space(PRESETS[preset].dots);
            let dark = PlateStops::of(PlateFamily::Neutral, Scheme::Dark)
                .tinted(PlateTint::Monochrome(tint));
            // The ink is near white, where the tint eases off.
            for (colour, least) in [(dark.base, 0.03), (dark.deep, 0.03), (dark.ink, 0.01)] {
                let got = hue_and_chroma(colour);
                assert!(
                    hue_gap(got.hue, tint.hue) < 6.0 && got.chroma > least,
                    "preset {preset}: {} is {got:?}, tint {tint:?}",
                    colour.css()
                );
            }
            let light = PlateStops::of(PlateFamily::Neutral, Scheme::Light)
                .tinted(PlateTint::Monochrome(tint));
            assert_eq!(light.base, Hex([255, 255, 255]), "white has no room");
            assert!(hue_gap(hue_and_chroma(light.deep).hue, tint.hue) < 20.0);
        }
    }

    #[test]
    fn muted_lowers_a_familys_chroma_and_keeps_its_hue() {
        // The dark paper: under `from_hex`'s 0.07 cap, so the drop is measurable.
        let paper = PlateStops::of(PlateFamily::Neutral, Scheme::Dark);
        let muted = paper.tinted(PlateTint::Muted);
        let (before, after) = (hue_and_chroma(paper.base), hue_and_chroma(muted.base));
        assert!(after.chroma < before.chroma, "{before:?} -> {after:?}");
        assert!(
            hue_gap(before.hue, after.hue) < 15.0,
            "{before:?} -> {after:?}"
        );
    }

    #[test]
    fn the_style_writes_both_schemes() {
        let tint = PlateTint::Monochrome(Tint {
            hue: 150.0,
            chroma: 0.06,
        });
        let style = tint_style(PlateFamily::Neutral, tint);
        let names: Vec<&str> = style
            .split(';')
            .filter_map(|pair| pair.split_once(':'))
            .map(|(name, _)| name)
            .collect();
        assert_eq!(
            names,
            [
                "--plate-base-l",
                "--plate-deep-l",
                "--plate-ink-l",
                "--plate-base-d",
                "--plate-deep-d",
                "--plate-ink-d"
            ]
        );
        let dark = PlateStops::of(PlateFamily::Neutral, Scheme::Dark).tinted(tint);
        assert!(style.contains(&format!("--plate-base-d:{};", dark.base.css())));
    }
}
