//! App icons for the gallery's launcher and dock specimens: coloured rounded squares, PNG-encoded,
//! standing in for the icon files an icon theme lookup returns.

use ds::{ExternalIcon, IconSize, IconSource, IconUrl};
use image::{ImageFormat, Rgba, RgbaImage};
use std::io::Cursor;

/// A 64 px app icon: a rounded square in `hue`'s colour, lighter at the top, with a white disc.
pub fn app_icon(hue: [u8; 3], size: IconSize) -> Option<IconSource> {
    let side = 64u32;
    let picture = RgbaImage::from_fn(side, side, |x, y| {
        let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
        let inside = rounded(fx, fy, side as f32, 14.0);
        let (dx, dy) = (fx - 32.0, fy - 34.0);
        let disc = dx * dx + dy * dy <= 11.0 * 11.0;
        let lift = 1.0 + 0.35 * (1.0 - fy / side as f32);
        let [r, g, b] = if disc {
            [250, 250, 250]
        } else {
            hue.map(|channel| (f32::from(channel) * lift).min(255.0) as u8)
        };
        Rgba([r, g, b, if inside { 255 } else { 0 }])
    });
    let mut bytes = Cursor::new(Vec::new());
    picture.write_to(&mut bytes, ImageFormat::Png).ok()?;
    Some(IconSource::Image(ExternalIcon {
        url: IconUrl::png(&bytes.into_inner()),
        size,
    }))
}

/// Whether `(x, y)` is inside a `side` square with corners of `radius`.
fn rounded(x: f32, y: f32, side: f32, radius: f32) -> bool {
    let cx = x.clamp(radius, side - radius);
    let cy = y.clamp(radius, side - radius);
    let (dx, dy) = (x - cx, y - cy);
    dx * dx + dy * dy <= radius * radius
}

/// Three apps: a browser, files and a terminal.
pub const APPS: [(&str, [u8; 3]); 3] = [
    ("Firefox", [214, 92, 38]),
    ("Files", [52, 120, 206]),
    ("Terminal", [58, 64, 72]),
];
