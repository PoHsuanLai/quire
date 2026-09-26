//! A vivid wallpaper and its compositor-blurred copy, for the Widget reference page's
//! "compositor blur" wall (design/23-WIDGETS.md section 1.1, M26-M30).
//!
//! The shell's widget card is a tint over the compositor's blur of the wallpaper; Blitz cannot
//! blur (spike S15), so the page lays a copy of this wallpaper, blurred here with the Gaussian
//! the reference fit implies (sigma 22 logical pixels), behind each card, clipped to it. The
//! wallpaper's colours are the reference screenshots': warm hue bands running down to the right
//! over a deep green on the left, soft yellow, orange and blue-grey fields on the right. Both
//! pictures are generated, so the gallery ships no image file.

use crate::data_uri;
use image::{ImageEncoder, Rgb, RgbImage, codecs::png::PngEncoder, imageops};
use std::sync::LazyLock;

/// The wall's size in logical pixels.
pub const WIDTH: u32 = 780;
/// The wall's height in logical pixels.
pub const HEIGHT: u32 = 430;

/// The sharp picture's pixels per logical pixel (a 2x snapshot stays crisp).
const SHARP_SCALE: u32 = 2;

/// The blurred copy is drawn at half the logical size and stretched back: a blur this wide
/// has nothing finer than a pixel at that size, and it keeps the Gaussian cheap.
const BLUR_DOWN: u32 = 2;

/// The fitted blur, as a Gaussian's standard deviation in logical pixels (M26).
pub const BLUR_SIGMA: f32 = 22.0;

/// The sharp wallpaper as a `data:` URI, made once.
pub fn sharp_uri() -> &'static str {
    static URI: LazyLock<String> = LazyLock::new(|| data_uri::png(&png(&sharp())));
    URI.as_str()
}

/// The blurred wallpaper as a `data:` URI, made once.
pub fn blurred_uri() -> &'static str {
    static URI: LazyLock<String> = LazyLock::new(|| data_uri::png(&png(&blurred())));
    URI.as_str()
}

/// The wallpaper at [`SHARP_SCALE`].
fn sharp() -> RgbImage {
    picture(SHARP_SCALE, 1)
}

/// The wallpaper at half size, blurred by [`BLUR_SIGMA`] scaled to that size.
fn blurred() -> RgbImage {
    let small = picture(1, BLUR_DOWN);
    imageops::blur(&small, BLUR_SIGMA / BLUR_DOWN as f32)
}

/// The wallpaper sampled `up` pixels per logical pixel, or one pixel per `down` logical pixels.
fn picture(up: u32, down: u32) -> RgbImage {
    let (width, height) = (WIDTH * up / down, HEIGHT * up / down);
    let step = down as f32 / up as f32;
    RgbImage::from_fn(width, height, |x, y| {
        let at = colour((x as f32 + 0.5) * step, (y as f32 + 0.5) * step);
        Rgb(at.map(|c| c.round().clamp(0.0, 255.0) as u8))
    })
}

/// The colour at logical `(x, y)`: the banded field on the left, the soft fields on the right,
/// a soft seam between them.
fn colour(x: f32, y: f32) -> [f32; 3] {
    let seam = ((x - 395.0) / 24.0).clamp(-1.0, 1.0) * 0.5 + 0.5;
    mix(bands(x, y), fields(x, y), seam)
}

/// Warm bands sloping down to the right (.36 of a pixel per pixel), sharp-edged: orange, with
/// a pink sky above it, over a deeper red band over green, as behind the reference's small
/// cards.
fn bands(x: f32, y: f32) -> [f32; 3] {
    let s = y - 0.36 * x;
    let orange = mix(
        [255.0, 128.0, 86.0],
        [255.0, 104.0, 88.0],
        (x / 380.0).clamp(0.0, 1.0),
    );
    if s < 71.0 {
        mix([236.0, 150.0, 190.0], orange, (s + 150.0) / 120.0)
    } else if s < 125.0 {
        mix([248.0, 78.0, 66.0], [240.0, 96.0, 74.0], (s - 71.0) / 54.0)
    } else {
        mix([0.0, 90.0, 36.0], [0.0, 74.0, 30.0], (s - 125.0) / 260.0)
    }
}

/// Soft fields: a golden glow at the top left, an orange one under it, a blue-grey light to
/// the right and a deep navy below, as behind the reference's medium battery card.
fn fields(x: f32, y: f32) -> [f32; 3] {
    let glow = |cx: f32, cy: f32, r: f32| {
        let (dx, dy) = ((x - cx) / r, (y - cy) / r);
        (-(dx * dx + dy * dy)).exp()
    };
    let base = mix(
        [206.0, 210.0, 216.0],
        [120.0, 140.0, 168.0],
        (x - 590.0) / 150.0,
    );
    let base = mix(base, [30.0, 40.0, 58.0], glow(700.0, 270.0, 130.0));
    let base = mix(base, [246.0, 150.0, 64.0], glow(430.0, 190.0, 110.0));
    mix(base, [252.0, 200.0, 70.0], glow(480.0, 50.0, 160.0))
}

fn mix(from: [f32; 3], to: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    [0, 1, 2].map(|c| from[c] + (to[c] - from[c]) * t)
}

/// `picture` as PNG bytes. Encoding an in-memory RGB buffer of the stated size cannot fail.
fn png(picture: &RgbImage) -> Vec<u8> {
    let mut bytes = Vec::new();
    let encoded = PngEncoder::new(&mut bytes).write_image(
        picture.as_raw(),
        picture.width(),
        picture.height(),
        image::ExtendedColorType::Rgb8,
    );
    match encoded {
        Ok(()) => bytes,
        Err(_) => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::{HEIGHT, SHARP_SCALE, WIDTH, blurred, blurred_uri, sharp};

    #[test]
    fn the_blurred_copy_softens_the_band_edges_the_sharp_one_keeps() {
        let sharp = sharp();
        assert_eq!(
            sharp.dimensions(),
            (WIDTH * SHARP_SCALE, HEIGHT * SHARP_SCALE)
        );
        // Down the column at logical x = 100 the green starts on one row in the sharp picture.
        let column = |y: u32| i32::from(sharp.get_pixel(200, y).0[0]);
        let jumps = (1..sharp.height())
            .filter(|&y| (column(y) - column(y - 1)).abs() > 120)
            .count();
        assert!(jumps >= 1, "a sharp band edge");
        let soft = blurred();
        let steepest = (1..soft.height())
            .map(|y| {
                (i32::from(soft.get_pixel(50, y).0[0]) - i32::from(soft.get_pixel(50, y - 1).0[0]))
                    .abs()
            })
            .max()
            .unwrap_or(0);
        assert!(steepest < 30, "no band edge survives the blur: {steepest}");
        assert!(blurred_uri().starts_with("data:image/png;base64,"));
    }
}
