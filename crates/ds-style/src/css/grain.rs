//! The frame's grain tile (design/03-COLOR.md section 8): the prototype's Park-Miller
//! generator, seed 7, one draw per pixel over 128 x 128, as a PNG `data:` URI the stylesheet
//! paints on `.ds-grain` (a renderer needs a `data:` net provider to load it).
//!
//! The prototype draws an opaque grey `v = floor(rnd x 255)` and blends it with `overlay`,
//! which Blitz cannot do. This tile is the "alpha noise" the plan names instead (open decision
//! 2, proposed mapping): a grey below the middle becomes black at alpha `255 - 2v`, one above
//! becomes white at alpha `2v - 255`. Painted normally, that is exactly `overlay` of the same
//! grey wherever the frame is darker than the middle in the black half, and wherever it is
//! lighter in the white half; the element's opacity (`--f-grain`) then sets the strength.

use ds_core::base64;
use ds_core::png::{self, Channels, Deflate, Raster};
use std::sync::LazyLock;

const SIZE: usize = 128;
const MODULUS: u64 = 2_147_483_647;
const MULTIPLIER: u64 = 16_807;
const SEED: u64 = 7;

/// The tile as a `data:image/png;base64,…` URI, built once.
pub fn grain_uri() -> &'static str {
    static URI: LazyLock<String> =
        LazyLock::new(|| format!("data:image/png;base64,{}", base64::encode(&grain_png())));
    URI.as_str()
}

/// The tile as a PNG: 8-bit grey with alpha, unfiltered, stored deflate (noise does not
/// compress).
fn grain_png() -> Vec<u8> {
    let pixels: Vec<u8> = greys().into_iter().flat_map(pixel).collect();
    png::encode(
        Raster {
            width: SIZE,
            height: SIZE,
            channels: Channels::GreyAlpha,
            bytes: &pixels,
        },
        Deflate::Stored,
    )
}

/// The prototype's greys, in raster order, computed the way its JavaScript does.
fn greys() -> Vec<u8> {
    let mut seed = SEED;
    (0..SIZE * SIZE)
        .map(|_| {
            seed = seed * MULTIPLIER % MODULUS;
            let rnd = seed as f64 / MODULUS as f64;
            // `Math.floor(rnd * 255)`: rnd is below 1, so this is 0..=254.
            (rnd * 255.0).floor() as u8
        })
        .collect()
}

/// A grey as black or white at an alpha: two bytes, grey then alpha.
fn pixel(grey: u8) -> [u8; 2] {
    let twice = i16::from(grey) * 2;
    if twice < 255 {
        [0, (255 - twice) as u8]
    } else {
        [255, (twice - 255) as u8]
    }
}

#[cfg(test)]
mod tests {
    use super::{grain_png, grain_uri};

    #[test]
    fn the_grain_is_the_prototypes_tile_as_alpha_noise() {
        assert!(grain_uri().starts_with("data:image/png;base64,iVBORw0KGgo"));
        let png = grain_png();
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(&png[12..16], b"IHDR");
        assert_eq!(&png[16..20], 128u32.to_be_bytes());
        assert_eq!(&png[20..24], 128u32.to_be_bytes());
        assert_eq!(&png[24..26], [8, 4], "8-bit grey with alpha");
        // The first scanline, from the stored deflate block after the zlib header: filter 0,
        // then the prototype's first draws. Seed 7: 7 x 16807 = 117649, floor(117649 / (2^31 -
        // 1) x 255) = 0, a full black; the second draw, 117649 x 16807 mod (2^31 - 1) =
        // 1977326743, is grey 234, white at alpha 2 x 234 - 255 = 213.
        let idat = png
            .windows(4)
            .position(|w| w == b"IDAT")
            .unwrap_or_else(|| panic!("no IDAT"));
        let raw = &png[idat + 4 + 2 + 5..];
        assert_eq!(&raw[..5], [0, 0, 255, 255, 213]);
    }
}
