//! A stand-in for the user's own photo (`UserPicture::Photo`), generated so the gallery ships no
//! image file: a portrait-shaped head and shoulders against a teal backdrop, taller than wide so
//! the round crop (`object-fit: cover`) has something to cut.

use crate::data_uri;
use crate::wallpaper::png;
use image::{Rgb, RgbImage};
use std::sync::LazyLock;

const WIDTH: u32 = 96;
const HEIGHT: u32 = 144;

/// The portrait as a `data:` URI, made once.
pub fn uri() -> &'static str {
    static URI: LazyLock<String> = LazyLock::new(|| data_uri::png(&png(&portrait())));
    URI.as_str()
}

/// A backdrop fading from pale to deeper teal, a round head and the shoulders under it.
fn portrait() -> RgbImage {
    RgbImage::from_fn(WIDTH, HEIGHT, |x, y| {
        let (u, v) = (x as f32, y as f32);
        let head = (u - 48.0).powi(2) + (v - 64.0).powi(2) < 21.0_f32.powi(2);
        let hair = (u - 48.0).powi(2) + (v - 58.0).powi(2) < 24.0_f32.powi(2) && v < 60.0;
        let shoulders = ((u - 48.0) / 44.0).powi(2) + ((v - 132.0) / 40.0).powi(2) < 1.0;
        match (hair, head, shoulders) {
            (true, _, _) => Rgb([74, 52, 44]),
            (_, true, _) => Rgb([226, 184, 150]),
            (_, _, true) => Rgb([52, 96, 140]),
            _ => {
                let t = v / HEIGHT as f32;
                let mix = |a: f32, b: f32| (a + (b - a) * t).round() as u8;
                Rgb([mix(150.0, 96.0), mix(196.0, 150.0), mix(200.0, 176.0)])
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{HEIGHT, WIDTH, portrait, uri};

    #[test]
    fn the_portrait_is_taller_than_wide_and_a_png() {
        let picture = portrait();
        assert_eq!((picture.width(), picture.height()), (WIDTH, HEIGHT));
        assert!(picture.height() > picture.width());
        assert!(uri().starts_with("data:image/png;base64,"));
    }
}
