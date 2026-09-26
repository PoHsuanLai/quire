//! The wallpaper the materials page lays its chrome over: a generated picture, so the gallery
//! ships no image file and writes no colour in CSS. Wide hue bands give the translucent tints
//! something to show through; a band of one-pixel stripes shows, at a glance, that nothing
//! behind a panel is blurred (spike S15: Blitz paints no `backdrop-filter`).

use crate::data_uri;
use ds::Scheme;
use image::{ImageEncoder, Rgb, RgbImage, codecs::png::PngEncoder};
use std::sync::LazyLock;

const WIDTH: u32 = 360;
const HEIGHT: u32 = 200;
const CALM_WIDTH: u32 = 480;
const CALM_HEIGHT: u32 = 300;

/// The wallpaper as a `data:` URI, made once.
pub fn uri() -> &'static str {
    static URI: LazyLock<String> = LazyLock::new(|| data_uri::png(&png(&picture())));
    URI.as_str()
}

/// A calm wallpaper for `scheme`, as a `data:` URI made once per scheme: a soft diagonal
/// two-stop gradient with one wide, blurred glow, the kind of desktop a widget is judged on
/// (the Widget looks page), where the hue bands above are a legibility test.
pub fn calm_uri(scheme: Scheme) -> &'static str {
    static LIGHT: LazyLock<String> = LazyLock::new(|| data_uri::png(&png(&calm(Scheme::Light))));
    static DARK: LazyLock<String> = LazyLock::new(|| data_uri::png(&png(&calm(Scheme::Dark))));
    match scheme {
        Scheme::Light => LIGHT.as_str(),
        Scheme::Dark => DARK.as_str(),
    }
}

/// The calm picture: `from` at the top left to `to` at the bottom right, and `glow` spread
/// round a point up and to the right.
fn calm(scheme: Scheme) -> RgbImage {
    let (from, to, glow) = match scheme {
        Scheme::Light => (
            [196.0, 211.0, 222.0],
            [233.0, 222.0, 208.0],
            [214.0, 204.0, 230.0],
        ),
        Scheme::Dark => ([22.0, 30.0, 42.0], [44.0, 34.0, 48.0], [38.0, 58.0, 70.0]),
    };
    let (width, height) = (CALM_WIDTH as f32, CALM_HEIGHT as f32);
    RgbImage::from_fn(CALM_WIDTH, CALM_HEIGHT, |x, y| {
        let (u, v) = (x as f32 / width, y as f32 / height);
        let t = ((u + v) / 2.0).clamp(0.0, 1.0);
        let (dx, dy) = (u - 0.72, (v - 0.28) * 0.8);
        let g = (-(dx * dx + dy * dy) / 0.06).exp() * 0.7;
        let channel = |c: usize| {
            let base = from[c] + (to[c] - from[c]) * t;
            (base + (glow[c] - base) * g).round().clamp(0.0, 255.0) as u8
        };
        Rgb([channel(0), channel(1), channel(2)])
    })
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
    use super::{calm, calm_uri, picture, uri};
    use ds::Scheme;

    #[test]
    fn the_calm_wallpaper_is_light_or_dark_with_its_scheme() {
        let luma = |scheme| {
            let p = calm(scheme).get_pixel(240, 150).0;
            u32::from(p[0]) + u32::from(p[1]) + u32::from(p[2])
        };
        assert!(luma(Scheme::Light) > 540 && luma(Scheme::Dark) < 180);
        assert_ne!(calm_uri(Scheme::Light), calm_uri(Scheme::Dark));
    }

    #[test]
    fn the_wallpaper_is_a_png_with_black_white_and_stripes() {
        let picture = picture();
        assert_eq!(picture.get_pixel(10, 5).0, [0, 0, 0]);
        assert_eq!(picture.get_pixel(10, 190).0, [255, 255, 255]);
        assert_ne!(picture.get_pixel(10, 80), picture.get_pixel(11, 80));
        assert!(uri().starts_with("data:image/png;base64,iVBORw0KGgo"));
    }
}
