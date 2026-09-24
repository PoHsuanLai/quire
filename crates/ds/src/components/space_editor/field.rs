//! The editor's hue x chroma field: the dot grid as two images built once per scheme, and the
//! arithmetic between a dot and its place on the field (design/03-COLOR.md section 9,
//! design/06-INTERACTIONS.md section 2.8). O-19: no canvas on Blitz, so the plane is images.
//!
//! S draws a 540 x 352 canvas, dots every 18 px, and scales it to the field (`S:870`), which
//! at S's own size is exactly half: 176 px tall, dots every 9 px. Scaled into a wider field the
//! same drawing stretches its dots into ellipses, so the field is two layers instead. The
//! colours are a small hue x chroma plane stretched to the field, where stretching a smooth
//! gradient changes nothing; over it lies one grid cell (18 x 18, drawn at 9 x 9 px) of the
//! ground with a round hole at its centre, tiled, so every dot stays round at any width and
//! shows the colour of its own place on the field.

use super::png;
use crate::appearance::Scheme;
use crate::space::{Dot, swatch};
use crate::tokens::Hex;
use std::sync::LazyLock;

/// The colour plane's size: one sample per grid cell of S's 540 x 352 drawing, stretched to the
/// field.
pub(super) const COLUMNS: usize = 60;
/// See [`COLUMNS`].
pub(super) const ROWS: usize = 39;
/// One grid cell of the drawing: dots every 18 px, starting half a step in (`S:1401`). The
/// tile is drawn at twice its CSS size, 9 px (`space_editor.css`), as S's canvas is.
pub(super) const STEP: usize = 18;
/// Each dot's radius (`S:1404`).
const RADIUS: f64 = 5.2;
/// Samples per pixel edge when a dot's rim crosses a pixel, standing in for the canvas's
/// anti-aliasing.
const SUBSAMPLES: u32 = 4;

/// The plane's ground (`S:1399`): identity pixels of an image, not theme tokens.
fn ground(scheme: Scheme) -> [u8; 3] {
    match scheme {
        Scheme::Light => [0xf3, 0xf4, 0xf1],
        Scheme::Dark => [0x1b, 0x1d, 0x1a],
    }
}

/// The dot a point of the field stands for: hue across, chroma down, both clamped.
pub(super) fn dot_at(x: f64, y: f64) -> Dot {
    Dot {
        hue: (x.clamp(0.0, 1.0) * 360.0) as f32,
        chroma: (1.0 - y.clamp(0.0, 1.0)) as f32,
    }
}

/// Where a dot sits on the field, as fractions across and down: the handle's `left` and `top`.
pub(super) fn place(dot: Dot) -> (f64, f64) {
    (
        f64::from(dot.hue) / 360.0,
        1.0 - f64::from(dot.chroma).clamp(0.0, 1.0),
    )
}

/// The colour at fractions `(x, y)` of the field, from the palette.
fn colour_at(x: f64, y: f64, scheme: Scheme) -> [u8; 3] {
    Hex::parse(&swatch(dot_at(x, y), scheme)).map_or(ground(scheme), |Hex(rgb)| rgb)
}

/// How much of pixel `(x, y)` the circle at `(cx, cy)` covers, 0 to 1.
fn coverage(x: usize, y: usize, cx: f64, cy: f64) -> f64 {
    let n = SUBSAMPLES;
    let inside = (0..n * n)
        .filter(|k| {
            let sx = x as f64 + (f64::from(k % n) + 0.5) / f64::from(n);
            let sy = y as f64 + (f64::from(k / n) + 0.5) / f64::from(n);
            (sx - cx).hypot(sy - cy) <= RADIUS
        })
        .count();
    inside as f64 / f64::from(n * n)
}

/// The colour plane's pixels, rows top to bottom: each the colour at its own centre.
fn colours(scheme: Scheme) -> Vec<[u8; 3]> {
    (0..ROWS)
        .flat_map(|y| (0..COLUMNS).map(move |x| (x, y)))
        .map(|(x, y)| {
            let across = (x as f64 + 0.5) / COLUMNS as f64;
            let down = (y as f64 + 0.5) / ROWS as f64;
            colour_at(across, down, scheme)
        })
        .collect()
}

/// The tile's pixels: the ground, opaque, with a hole where the cell's dot is.
fn tile(scheme: Scheme) -> Vec<[u8; 4]> {
    let [r, g, b] = ground(scheme);
    let centre = STEP as f64 / 2.0;
    (0..STEP)
        .flat_map(|y| (0..STEP).map(move |x| (x, y)))
        .map(|(x, y)| {
            // The coverage is 0 to 1, so the alpha stays within 0..=255.
            let alpha = ((1.0 - coverage(x, y, centre, centre)) * 255.0).round() as u8;
            [r, g, b, alpha]
        })
        .collect()
}

/// The field's two images for `scheme`, as `data:` URIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Plane {
    /// The hue x chroma colours, stretched to the field.
    pub(super) colours: &'static str,
    /// One cell of the ground with its dot's hole, tiled.
    pub(super) dots: &'static str,
}

/// The images for `scheme`, built on first use and kept.
pub(super) fn plane(scheme: Scheme) -> Plane {
    static LIGHT: LazyLock<(String, String)> = LazyLock::new(|| uris(Scheme::Light));
    static DARK: LazyLock<(String, String)> = LazyLock::new(|| uris(Scheme::Dark));
    let (colours, dots) = match scheme {
        Scheme::Light => &*LIGHT,
        Scheme::Dark => &*DARK,
    };
    Plane { colours, dots }
}

fn uris(scheme: Scheme) -> (String, String) {
    let colours = png::rgb(COLUMNS, ROWS, &colours(scheme));
    let dots = png::rgba(STEP, STEP, &tile(scheme));
    (uri(&colours), uri(&dots))
}

fn uri(png: &[u8]) -> String {
    format!("data:image/png;base64,{}", png::base64(png))
}

#[cfg(test)]
mod tests {
    use super::{coverage, dot_at, place};
    use crate::space::Dot;

    #[test]
    fn a_dot_and_its_place_round_trip() {
        const CASES: &[(f32, f32, f64, f64)] = &[
            (268.0, 0.72, 268.0 / 360.0, 0.28),
            (0.0, 1.0, 0.0, 0.0),
            (180.0, 0.0, 0.5, 1.0),
        ];
        for &(hue, chroma, left, top) in CASES {
            let (x, y) = place(Dot { hue, chroma });
            assert!(
                (x - left).abs() < 1e-6 && (y - top).abs() < 1e-6,
                "{hue} {chroma}"
            );
            let back = dot_at(x, y);
            assert!((back.hue - hue).abs() < 1e-3, "{hue}: {}", back.hue);
            assert!(
                (back.chroma - chroma).abs() < 1e-6,
                "{chroma}: {}",
                back.chroma
            );
        }
        let outside = dot_at(1.4, -0.2);
        assert_eq!((outside.hue, outside.chroma), (360.0, 1.0), "clamped");
    }

    #[test]
    fn coverage_is_full_inside_and_empty_outside() {
        assert_eq!(coverage(9, 9, 9.0, 9.0), 1.0);
        assert_eq!(coverage(20, 9, 9.0, 9.0), 0.0);
        let rim = coverage(14, 9, 9.0, 9.0);
        assert!(rim > 0.0 && rim < 1.0, "the rim is partial: {rim}");
    }
}
