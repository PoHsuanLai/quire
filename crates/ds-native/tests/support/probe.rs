//! Probes shared by the headless tests: a laid-out rect, the pixels inside it, and how far
//! they stand out from their own ground. Each test crate uses some of them.

#![allow(dead_code)]

use ds::Rect;
use ds_native::Harness;
use image::RgbaImage;

pub fn rect(harness: &Harness, selector: &str) -> Rect {
    harness
        .rect(selector)
        .unwrap_or_else(|| panic!("{selector} is not in the document:\n{}", harness.html()))
}

/// Where `part` is painted inside a pill centred by `translateX(-50%)`: the layout rect Blitz
/// reports leaves the transform out, so the part sits half the pill's width further left.
pub fn centred(harness: &Harness, pill: &str, part: &str) -> Rect {
    let shift = rect(harness, pill).size.width.0 / 2.0;
    let mut at = rect(harness, part);
    at.origin.x = ds::Px(at.origin.x.0 - shift);
    at
}

/// Keep `frame` in the test's scratch directory, for review.
pub fn keep(frame: &RgbaImage, name: &str) {
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("{name}.png"));
    frame.save(&path).ok();
}

/// The pixels inside `rect`, inset by `inset` on every side.
pub fn pixels(frame: &RgbaImage, rect: Rect, inset: f32) -> Vec<[u8; 4]> {
    let x0 = (rect.origin.x.0 + inset).max(0.0) as u32;
    let y0 = (rect.origin.y.0 + inset).max(0.0) as u32;
    let x1 = ((rect.origin.x.0 + rect.size.width.0 - inset) as u32).min(frame.width());
    let y1 = ((rect.origin.y.0 + rect.size.height.0 - inset) as u32).min(frame.height());
    (y0..y1)
        .flat_map(|y| (x0..x1).map(move |x| (x, y)))
        .map(|(x, y)| frame.get_pixel(x, y).0)
        .collect()
}

/// The most common colour among `seen`.
pub fn modal(seen: &[[u8; 4]]) -> [u8; 4] {
    let mut sorted = seen.to_vec();
    sorted.sort_unstable();
    sorted
        .chunk_by(|a, b| a == b)
        .max_by_key(|run| run.len())
        .map_or([0; 4], |run| run[0])
}

/// How far apart two colours are: the largest channel difference.
pub fn distance(a: [u8; 4], b: [u8; 4]) -> u8 {
    (0..3).map(|i| a[i].abs_diff(b[i])).max().unwrap_or(0)
}

/// How many of `seen` stand out from their region's own ground by more than `by`.
pub fn ink(seen: &[[u8; 4]], by: u8) -> usize {
    let ground = modal(seen);
    seen.iter()
        .filter(|pixel| distance(**pixel, ground) > by)
        .count()
}
