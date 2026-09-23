//! The wallpaper the materials page lays its chrome over: a generated picture, so the gallery
//! ships no image file and writes no colour in CSS. Wide hue bands give the translucent tints
//! something to show through; a band of one-pixel stripes shows, at a glance, that nothing
//! behind a panel is blurred (spike S15: Blitz paints no `backdrop-filter`).

use crate::data_uri;
use image::{ImageEncoder, Rgb, RgbImage, codecs::png::PngEncoder};
use std::sync::LazyLock;

const WIDTH: u32 = 360;
const HEIGHT: u32 = 200;

/// The wallpaper as a `data:` URI, made once.
pub fn uri() -> &'static str {
    static URI: LazyLock<String> = LazyLock::new(|| data_uri::png(&png(&picture())));
    URI.as_str()
}

/// The picture: diagonal hue bands, a black and a white bar, and a striped band.
fn picture() -> RgbImage {
    RgbImage::from_fn(WIDTH, HEIGHT, |x, y| {
        let stripes = (70..100).contains(&y);
        match (x, y) {
            (_, 0..=23) => Rgb([0, 0, 0]),
            (_, 176..) => Rgb([255, 255, 255]),
            _ if stripes && x % 2 == 0 => Rgb([0, 0, 0]),
            _ if stripes => Rgb([255, 255, 255]),
            _ => band(x + y),
        }
    })
}

/// A saturated hue for position `t` along the diagonal.
fn band(t: u32) -> Rgb<u8> {
    let hue = (t % 240) as f32 / 240.0 * 6.0;
    let rising = (hue.fract() * 255.0).round() as u8;
    let falling = 255 - rising;
    let [r, g, b] = match hue as u32 {
        0 => [255, rising, 40],
        1 => [falling, 255, 40],
        2 => [40, 255, rising],
        3 => [40, falling, 255],
        4 => [rising, 40, 255],
        _ => [255, 40, falling],
    };
    Rgb([r, g, b])
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
    use super::{picture, uri};

    #[test]
    fn the_wallpaper_is_a_png_with_black_white_and_stripes() {
        let picture = picture();
        assert_eq!(picture.get_pixel(10, 5).0, [0, 0, 0]);
        assert_eq!(picture.get_pixel(10, 190).0, [255, 255, 255]);
        assert_ne!(picture.get_pixel(10, 80), picture.get_pixel(11, 80));
        assert!(uri().starts_with("data:image/png;base64,iVBORw0KGgo"));
    }
}
