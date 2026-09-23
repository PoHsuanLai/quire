//! The editor's hue x chroma field: the dot-grid plane as a PNG built once per scheme, and the
//! arithmetic between a dot and its place on the field (design/03-COLOR.md section 9,
//! design/06-INTERACTIONS.md section 2.8). O-19: no canvas on Blitz, so the plane is an image.

use super::png;
use crate::appearance::Scheme;
use crate::space::{Dot, swatch};
use crate::tokens::Hex;
use std::sync::LazyLock;

/// The drawing's size; the field scales it to fit (`S:870`).
pub(super) const WIDTH: usize = 540;
/// See [`WIDTH`].
pub(super) const HEIGHT: usize = 352;
/// Dots every 18 px, starting half a step in (`S:1401`).
const STEP: usize = 18;
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

/// The colour of the grid dot centred at `(cx, cy)` in drawing pixels, from the palette.
fn dot_colour(cx: usize, cy: usize, scheme: Scheme) -> [u8; 3] {
    let dot = dot_at(cx as f64 / WIDTH as f64, cy as f64 / HEIGHT as f64);
    Hex::parse(&swatch(dot, scheme)).map_or(ground(scheme), |Hex(rgb)| rgb)
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

fn blend(under: [u8; 3], over: [u8; 3], alpha: f64) -> [u8; 3] {
    // Each channel stays within 0..=255 by construction, so the cast cannot truncate.
    std::array::from_fn(|i| {
        (f64::from(under[i]) * (1.0 - alpha) + f64::from(over[i]) * alpha).round() as u8
    })
}

/// The plane's pixels, rows top to bottom: the ground with a disc at every grid point.
fn pixels(scheme: Scheme) -> Vec<[u8; 3]> {
    let floor = ground(scheme);
    let mut out = vec![floor; WIDTH * HEIGHT];
    let reach = RADIUS.ceil() as usize + 1;
    for cy in (STEP / 2..HEIGHT).step_by(STEP) {
        for cx in (STEP / 2..WIDTH).step_by(STEP) {
            let colour = dot_colour(cx, cy, scheme);
            for y in cy.saturating_sub(reach)..(cy + reach).min(HEIGHT) {
                for x in cx.saturating_sub(reach)..(cx + reach).min(WIDTH) {
                    let alpha = coverage(x, y, cx as f64, cy as f64);
                    if alpha > 0.0 {
                        out[y * WIDTH + x] = blend(floor, colour, alpha);
                    }
                }
            }
        }
    }
    out
}

/// The plane for `scheme` as a `data:` URI, built on first use and kept.
pub(super) fn plane(scheme: Scheme) -> &'static str {
    static LIGHT: LazyLock<String> = LazyLock::new(|| uri(Scheme::Light));
    static DARK: LazyLock<String> = LazyLock::new(|| uri(Scheme::Dark));
    match scheme {
        Scheme::Light => &LIGHT,
        Scheme::Dark => &DARK,
    }
}

fn uri(scheme: Scheme) -> String {
    let png = png::rgb(WIDTH, HEIGHT, &pixels(scheme));
    format!("data:image/png;base64,{}", png::base64(&png))
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
