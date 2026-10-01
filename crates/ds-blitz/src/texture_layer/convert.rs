//! Uploaded pixels as the GPU holds them: premultiplied RGBA8, the one alpha the renderer
//! composites (its texture sampler reads a texel as premultiplied and blends it over what is
//! beneath). Pure.

use super::model::{PixelFormat, Pixels};
use std::borrow::Cow;

/// `pixels` as premultiplied RGBA8: premultiplied RGBA as it is (borrowed), RGB with an opaque
/// alpha, straight RGBA with each colour channel multiplied by its alpha and rounded to nearest.
pub(crate) fn premultiplied<'a>(pixels: &Pixels<'a>) -> Cow<'a, [u8]> {
    let bytes = pixels.bytes();
    match pixels.format() {
        PixelFormat::Rgba8Premultiplied => Cow::Borrowed(bytes),
        PixelFormat::Rgb8 => Cow::Owned(
            bytes
                .chunks_exact(3)
                .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], u8::MAX])
                .collect(),
        ),
        PixelFormat::Rgba8Straight => Cow::Owned(
            bytes
                .chunks_exact(4)
                .flat_map(|rgba| {
                    let alpha = u16::from(rgba[3]);
                    let scaled = |channel: u8| ((u16::from(channel) * alpha + 127) / 255) as u8;
                    [scaled(rgba[0]), scaled(rgba[1]), scaled(rgba[2]), rgba[3]]
                })
                .collect(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{PixelFormat, Pixels, premultiplied};

    #[test]
    fn each_format_becomes_premultiplied_rgba() {
        // (name, format, bytes, premultiplied)
        const CASES: &[(&str, PixelFormat, &[u8], &[u8])] = &[
            (
                "premultiplied passes through",
                PixelFormat::Rgba8Premultiplied,
                &[10, 20, 30, 40],
                &[10, 20, 30, 40],
            ),
            (
                "rgb gains an opaque alpha",
                PixelFormat::Rgb8,
                &[1, 2, 3, 4, 5, 6],
                &[1, 2, 3, 255, 4, 5, 6, 255],
            ),
            (
                "straight opaque is unchanged",
                PixelFormat::Rgba8Straight,
                &[200, 100, 50, 255],
                &[200, 100, 50, 255],
            ),
            (
                "straight half alpha halves the colour, rounding up",
                PixelFormat::Rgba8Straight,
                &[255, 255, 255, 128],
                &[128, 128, 128, 128],
            ),
            (
                "straight transparent is black",
                PixelFormat::Rgba8Straight,
                &[255, 128, 7, 0],
                &[0, 0, 0, 0],
            ),
        ];
        for (name, format, bytes, want) in CASES {
            let width = (bytes.len() / format.bytes_per_pixel()) as u32;
            let pixels = Pixels::new(*format, width, 1, bytes).expect("a valid picture");
            assert_eq!(&*premultiplied(&pixels), *want, "{name}");
        }
    }
}
