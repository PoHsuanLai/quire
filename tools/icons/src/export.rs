use image::{Rgba, Rgba32FImage, imageops::FilterType, imageops::resize};

use crate::{
    PlateGrid, Template, compose::Shadow, drop_shadow, linear_to_srgb, plate_mask, srgb_to_linear,
};

/// The freedesktop hicolor sizes (design/08-ICONS.md 2.6, settled); the master is 1024.
pub const EXPORT_SIZES: [u32; 8] = [16, 24, 32, 48, 64, 128, 256, 512];

/// One rendered size.
#[derive(Debug, Clone, PartialEq)]
pub struct Export {
    pub size: u32,
    pub image: Rgba32FImage,
}

/// The plate grid of an export size (re-exported for callers that place things on it).
pub fn grid_for(size: u32, t: &Template) -> PlateGrid {
    PlateGrid::for_canvas(size, t)
}

/// One export size from the 1024 `flat` master (08 2.6): the plate square is cut out,
/// downscaled with Lanczos3 in linear light, premultiplied, placed on the size's own grid, and
/// the size's own squircle becomes its alpha (a masked edge is never downscaled). The shadow
/// is added only when asked and only at 48 px and up.
pub fn export(flat: &Rgba32FImage, t: &Template, size: u32, shadow: Shadow) -> Export {
    let from = PlateGrid::for_canvas(flat.width(), t);
    let to = PlateGrid::for_canvas(size, t);
    let crop = Rgba32FImage::from_fn(from.side, from.side, |x, y| {
        to_linear_premul(flat.get_pixel(x + from.origin, y + from.origin).0)
    });
    let small = resize(&crop, to.side, to.side, FilterType::Lanczos3);
    let mask = plate_mask(to, t);
    let placed = Rgba32FImage::from_fn(size, size, |x, y| {
        let inside =
            x >= to.origin && y >= to.origin && x < to.origin + to.side && y < to.origin + to.side;
        let a = mask.at(i64::from(x), i64::from(y));
        match inside {
            true if a > 0.0 => {
                let p = from_linear_premul(small.get_pixel(x - to.origin, y - to.origin).0);
                Rgba([p[0], p[1], p[2], a])
            }
            _ => Rgba([0.0; 4]),
        }
    });
    let image = match shadow {
        Shadow::Baked if size >= 48 => drop_shadow(&placed, to, t),
        _ => placed,
    };
    Export { size, image }
}

fn to_linear_premul(p: [f32; 4]) -> Rgba<f32> {
    Rgba([
        srgb_to_linear(p[0]) * p[3],
        srgb_to_linear(p[1]) * p[3],
        srgb_to_linear(p[2]) * p[3],
        p[3],
    ])
}

fn from_linear_premul(p: [f32; 4]) -> [f32; 3] {
    let a = p[3].max(1e-6);
    [
        linear_to_srgb(p[0] / a),
        linear_to_srgb(p[1] / a),
        linear_to_srgb(p[2] / a),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Family, compose};

    #[test]
    fn every_size_keeps_the_plate_silhouette() {
        let t = Template::default();
        let grid = PlateGrid::for_canvas(1024, &t);
        let flat = compose(
            grid,
            Family::Green.stops(),
            &Rgba32FImage::new(1024, 1024),
            &t,
        );
        for size in EXPORT_SIZES {
            let e = export(&flat, &t, size, Shadow::Omitted);
            assert_eq!(e.image.dimensions(), (size, size), "{size}");
            let mask = plate_mask(PlateGrid::for_canvas(size, &t), &t);
            let alpha: Vec<f32> = e.image.pixels().map(|p| p.0[3]).collect();
            assert_eq!(alpha, mask.data, "{size}: alpha is the size's own squircle");
            let c = e.image.get_pixel(size / 2, size / 2).0;
            assert!(
                c[1] > c[0] && c[1] > c[2],
                "{size}: green plate stays green: {c:?}"
            );
        }
    }
}
