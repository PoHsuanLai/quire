//! The design's grain (03-COLOR 8): a 128 px grey-noise tile from a Park-Miller generator with
//! seed 7, blended with `overlay` at `grain / 100 x 0.20` (light).

use image::{Rgba, Rgba32FImage};

use crate::{Grain, Plane};

/// Tile side in px (03-COLOR 8, `S:87`).
pub const GRAIN_TILE: u32 = 128;

/// The grey tile, 0..=1, exactly the prototype's sequence (`v = floor(rnd * 255)`).
pub fn grain_tile() -> Plane {
    let mut seed: u64 = 7;
    let values = (0..GRAIN_TILE * GRAIN_TILE)
        .map(|_| {
            seed = (seed * 16_807) % 2_147_483_647;
            let rnd = seed as f64 / 2_147_483_647.0;
            (rnd * 255.0).floor() as f32 / 255.0
        })
        .collect();
    Plane {
        width: GRAIN_TILE,
        height: GRAIN_TILE,
        data: values,
    }
}

fn overlay(b: f32, s: f32) -> f32 {
    match b < 0.5 {
        true => 2.0 * b * s,
        false => 1.0 - 2.0 * (1.0 - b) * (1.0 - s),
    }
}

/// `img` with the grain tile overlaid at the light-theme opacity for `grain`.
pub fn apply_grain(img: &Rgba32FImage, grain: Grain, tile: &Plane) -> Rgba32FImage {
    let opacity = f32::from(grain.0.min(100)) / 100.0 * 0.20;
    Rgba32FImage::from_fn(img.width(), img.height(), |x, y| {
        let p = img.get_pixel(x, y).0;
        let s = tile.at(i64::from(x % GRAIN_TILE), i64::from(y % GRAIN_TILE));
        let mix = |b: f32| b + (overlay(b, s) - b) * opacity;
        Rgba([mix(p[0]), mix(p[1]), mix(p[2]), p[3]])
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The first values of the prototype's generator from seed 7 (computed in JS-equivalent
    /// double precision): 0, 234, 73.
    #[test]
    fn tile_matches_the_prototype_sequence() {
        let t = grain_tile();
        let want = [0.0, 234.0, 73.0].map(|v: f32| v / 255.0);
        assert_eq!(&t.data[..3], &want);
    }

    #[test]
    fn zero_grain_is_identity_and_mean_is_neutral() {
        let img = Rgba32FImage::from_pixel(128, 128, Rgba([0.4, 0.6, 0.2, 1.0]));
        let tile = grain_tile();
        assert_eq!(apply_grain(&img, Grain(0), &tile), img);
        let g = apply_grain(&img, Grain(100), &tile);
        let mean = g.pixels().map(|p| p.0[1]).sum::<f32>() / (128.0 * 128.0);
        assert!(
            (mean - 0.6).abs() < 0.02,
            "overlay of mid noise keeps the mean: {mean}"
        );
    }
}
