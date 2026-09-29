//! The dark plate fix F3 as a table (design/29-SIZING.md section 11): under Monochrome (neutral,
//! the Work and the Home tint) and Muted, the dark neutral plate stands about 3:1 off the dark
//! dock, its glyph reads on it, and a raster's darkest ink is lifted above the plate.

use super::family::PlateFamily;
use super::plate_tint::{PlateStops, PlateTint};
use super::retint::{IconStyle, Tint, oklab, retint_in};
use crate::appearance::theme::Scheme;
use crate::space::{contrast::ratio, presets::PRESETS};
use crate::tokens::hex::Hex;

/// The dark dock the sizing mockups measured against (design/29-SIZING.md section 11).
const DOCK_DARK: &str = "#2A2C30";
/// A third-party raster's darkest ink: Chrome's ring, a black logo.
const RASTER_DARK: [u8; 4] = [0x20, 0x21, 0x24, 255];

fn lightness(colour: Hex) -> f64 {
    oklab(colour.0)[0]
}

#[test]
fn a_dark_tinted_plate_clears_the_dock_and_its_art_clears_the_plate() {
    let tints = [
        PlateTint::Muted,
        PlateTint::Monochrome(Tint::NEUTRAL),
        PlateTint::Monochrome(Tint::space(PRESETS[0].dots)),
        PlateTint::Monochrome(Tint::space(PRESETS[1].dots)),
    ];
    for tint in tints {
        let plate =
            PlateStops::of(PlateFamily::Neutral, Scheme::Dark).tinted_in(tint, Scheme::Dark);
        let dock = ratio(&plate.base.css(), DOCK_DARK).expect("hex colours");
        assert!(
            dock >= 2.9,
            "{tint:?}: plate {} is {dock:.2}:1 on the dock",
            plate.base.css()
        );
        for stop in [plate.base, plate.deep] {
            let l = lightness(stop);
            assert!(
                (0.455..=0.565).contains(&l),
                "{tint:?}: stop {} at L {l:.3}",
                stop.css()
            );
        }
        let ink = ratio(&plate.ink.css(), &plate.base.css()).expect("hex colours");
        assert!(ink >= 3.0, "{tint:?}: glyph {ink:.2}:1 on the plate");
        let (style, tone) = match tint {
            PlateTint::Muted => (IconStyle::Muted, Tint::NEUTRAL),
            PlateTint::Monochrome(tint) => (IconStyle::Monochrome, tint),
        };
        let mut raster = RASTER_DARK.to_vec();
        retint_in(&mut raster, style, tone, Scheme::Dark);
        let lifted = lightness(Hex([raster[0], raster[1], raster[2]]));
        assert!(
            lifted >= 0.62 - 0.01 && lifted > lightness(plate.base),
            "{tint:?}: the raster's dark ink sits at L {lifted:.3}"
        );
    }
}

#[test]
fn the_light_scheme_is_unchanged() {
    let tint = PlateTint::Monochrome(Tint::space(PRESETS[0].dots));
    let light = PlateStops::of(PlateFamily::Neutral, Scheme::Light);
    assert_eq!(light.tinted_in(tint, Scheme::Light), light.tinted(tint));
    let mut before = RASTER_DARK.to_vec();
    let mut after = RASTER_DARK.to_vec();
    crate::icon::retint::retint(&mut before, IconStyle::Muted, Tint::NEUTRAL);
    retint_in(&mut after, IconStyle::Muted, Tint::NEUTRAL, Scheme::Light);
    assert_eq!(before, after);
}
