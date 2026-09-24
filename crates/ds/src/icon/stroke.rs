//! A glyph's stroke width, snapped to whole device pixels at a fractional scale
//! (design/08-ICONS.md section 1.4.1).
//!
//! The stroke is 2 in the 24 grid, so a 16 px glyph draws 1.33 logical pixels: 2.0 device pixels
//! at 1.5 (exact), but 1.67 at 1.25 and 2.33 at 1.75, whose edges land part-way through a
//! device pixel and blur. At a fractional scale the width is rounded to the nearest *even* number
//! of device pixels (never below one) and written back in grid units: Lucide draws on whole grid
//! lines, and an even stroke centred on a grid line that falls on a device pixel boundary has
//! both edges on boundaries too (an odd one would sit half a pixel off). At a whole scale, 1x
//! included, it stays the design's 2, so every picture at 1x and 2x is what it was.

use super::render::IconSize;
use crate::geometry::{Grid, Scale};

/// The design stroke in grid units.
const DESIGN: u64 = 2;
/// The glyph grid's side.
const GRID: u64 = 24;

/// The `stroke-width` attribute a `size` glyph writes at `scale`: `2`, or the snapped width in
/// grid units to four decimals (`2.4` for a 16 px glyph at 1.25).
pub fn stroke_width(size: IconSize, scale: Scale) -> String {
    match scale.grid() {
        Grid::Whole => DESIGN.to_string(),
        Grid::Fractional => snapped(size, scale),
    }
}

/// How many whole device pixels a snapped stroke covers at `scale`: the design width
/// (`2 * side / 24 * n / 120`) rounded to the nearest even count, halves up, never below one.
pub fn stroke_device_pixels(size: IconSize, scale: Scale) -> u64 {
    let side = u64::from(size.px());
    let per = GRID * u64::from(Scale::DENOMINATOR);
    let exact = DESIGN * side * u64::from(scale.numerator());
    (2 * ((exact + per) / (2 * per))).max(1)
}

fn snapped(size: IconSize, scale: Scale) -> String {
    const PLACES: u64 = 10_000;
    let device = stroke_device_pixels(size, scale);
    // Back to grid units: device * 24 * 120 / (side * n).
    let numerator = device * GRID * u64::from(Scale::DENOMINATOR) * PLACES;
    let denominator = u64::from(size.px()).max(1) * u64::from(scale.numerator());
    let scaled = (numerator + denominator / 2) / denominator;
    let (whole, part) = (scaled / PLACES, scaled % PLACES);
    match part {
        0 => whole.to_string(),
        _ => format!("{whole}.{}", format!("{part:04}").trim_end_matches('0')),
    }
}

#[cfg(test)]
mod tests {
    use super::{stroke_device_pixels, stroke_width};
    use crate::geometry::Scale;
    use crate::icon::render::{IconPx, IconSize};

    /// A size and fractional scale, the attribute and the device pixels it covers.
    const CASES: &[(IconSize, Scale, &str, u64)] = &[
        (IconSize::Base, Scale(150), "2.4", 2),
        (IconSize::Base, Scale(180), "2", 2),
        (IconSize::Base, Scale(210), "1.7143", 2),
        (IconSize::Px(IconPx(24)), Scale(150), "1.6", 2),
        (IconSize::Px(IconPx(24)), Scale(180), "2.6667", 4),
        (IconSize::Px(IconPx(24)), Scale(210), "2.2857", 4),
        (IconSize::Tile, Scale(180), "1.8824", 2),
        (IconSize::Tile48, Scale(180), "2", 6),
    ];

    #[test]
    fn the_stroke_is_an_even_number_of_device_pixels_at_a_fractional_scale() {
        for &(size, scale, attribute, device) in CASES {
            assert_eq!(
                stroke_width(size, scale),
                attribute,
                "{size:?} at {scale:?}"
            );
            assert_eq!(
                stroke_device_pixels(size, scale),
                device,
                "{size:?} at {scale:?}"
            );
        }
    }

    #[test]
    fn a_whole_scale_keeps_the_design_stroke() {
        for scale in [Scale::ONE, Scale(240), Scale(360)] {
            for size in [IconSize::Base, IconSize::Tile, IconSize::Px(IconPx(24))] {
                assert_eq!(stroke_width(size, scale), "2", "{size:?} at {scale:?}");
            }
        }
    }
}
