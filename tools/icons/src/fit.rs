use image::{Rgba, Rgba32FImage};

use crate::{IconsError, Plane, PlateGrid, Template};

/// An axis-aligned box in pixels, `x0..x1` by `y0..y1` (exclusive ends).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

impl Rect {
    pub fn width(self) -> f32 {
        self.x1 - self.x0
    }
    pub fn height(self) -> f32 {
        self.y1 - self.y0
    }
}

/// The bounding box of pixels with alpha above `min` (08 2.4 counts alpha > 0.1), if any.
pub fn alpha_bbox(alpha: &Plane, min: f32) -> Option<Rect> {
    let w = alpha.width as usize;
    let hits = alpha
        .data
        .iter()
        .enumerate()
        .filter(|(_, a)| **a > min)
        .map(|(i, _)| ((i % w) as f32, (i / w) as f32));
    hits.fold(None, |acc: Option<Rect>, (x, y)| {
        Some(match acc {
            None => Rect {
                x0: x,
                y0: y,
                x1: x + 1.0,
                y1: y + 1.0,
            },
            Some(r) => Rect {
                x0: r.x0.min(x),
                y0: r.y0.min(y),
                x1: r.x1.max(x + 1.0),
                y1: r.y1.max(y + 1.0),
            },
        })
    })
}

fn centroid(alpha: &Plane) -> (f32, f32) {
    let w = alpha.width as usize;
    let (sx, sy, sa) = alpha
        .data
        .iter()
        .enumerate()
        .fold((0.0, 0.0, 0.0), |(sx, sy, sa), (i, &a)| {
            (sx + a * (i % w) as f32, sy + a * (i / w) as f32, sa + a)
        });
    (sx / sa + 0.5, sy / sa + 0.5)
}

/// Places a keyed layer on a transparent canvas of `canvas` px (08 2.4). The object's alpha
/// decides the placement: its longer axis fills `fill` of the plate side, its centroid sits
/// `lift` above the plate centre, pushed back inside the safe square if that moved it out.
/// The whole layer (object and its shadow matte) moves with it; the shadow is then cut at
/// the safe square with a 4 % feather.
pub fn fit_object(
    layer: &Rgba32FImage,
    alpha: &Plane,
    canvas: u32,
    t: &Template,
) -> Result<Rgba32FImage, IconsError> {
    let bbox = alpha_bbox(alpha, 0.1).ok_or(IconsError::EmptyObject)?;
    let grid = PlateGrid::for_canvas(canvas, t);
    let side = grid.side as f32;
    let scale = t.fill.0 * side / bbox.width().max(bbox.height());
    let (cx, cy) = centroid(alpha);
    let centre = grid.origin as f32 + side / 2.0;
    let safe = Rect {
        x0: centre - t.safe.0 * side / 2.0,
        y0: centre - t.safe.0 * side / 2.0,
        x1: centre + t.safe.0 * side / 2.0,
        y1: centre + t.safe.0 * side / 2.0,
    };
    let tx = clamp_shift(
        centre - cx * scale,
        bbox.x0 * scale,
        bbox.x1 * scale,
        safe.x0,
        safe.x1,
    );
    let ty = clamp_shift(
        centre - t.lift.0 * side - cy * scale,
        bbox.y0 * scale,
        bbox.y1 * scale,
        safe.y0,
        safe.y1,
    );
    let placed = resample(layer, canvas, scale, (tx, ty));
    let feather = 0.04 * side;
    let window =
        |v: f32, lo: f32, hi: f32| ((v - lo) / feather).min((hi - v) / feather).clamp(0.0, 1.0);
    Ok(Rgba32FImage::from_fn(canvas, canvas, |x, y| {
        let p = placed.get_pixel(x, y).0;
        let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
        Rgba([
            p[0],
            p[1],
            p[2],
            p[3] * window(fx, safe.x0 - feather, safe.x1 + feather).min(window(
                fy,
                safe.y0 - feather,
                safe.y1 + feather,
            )),
        ])
    }))
}

/// The translation nearest to `t` that keeps `lo + t .. hi + t` inside `min..max`.
fn clamp_shift(t: f32, lo: f32, hi: f32, min: f32, max: f32) -> f32 {
    if hi - lo > max - min {
        return (min + max) / 2.0 - (lo + hi) / 2.0;
    }
    t.max(min - lo).min(max - hi)
}

/// Area-weighted resample (box filter over the source footprint), premultiplied.
fn resample(rgb: &Rgba32FImage, canvas: u32, scale: f32, (tx, ty): (f32, f32)) -> Rgba32FImage {
    let (w, h) = rgb.dimensions();
    let step = 1.0 / scale;
    let taps = step.ceil().max(1.0) as u32 * 2;
    Rgba32FImage::from_fn(canvas, canvas, |x, y| {
        let (sx, sy) = ((x as f32 - tx) * step, (y as f32 - ty) * step);
        if sx < -step || sy < -step || sx >= w as f32 || sy >= h as f32 {
            return Rgba([0.0; 4]);
        }
        let mut acc = [0.0f32; 4];
        for j in 0..taps {
            for i in 0..taps {
                let px = (sx + (i as f32 + 0.5) * step / taps as f32).floor() as i64;
                let py = (sy + (j as f32 + 0.5) * step / taps as f32).floor() as i64;
                if px < 0 || py < 0 || px >= i64::from(w) || py >= i64::from(h) {
                    continue;
                }
                let p = rgb.get_pixel(px as u32, py as u32).0;
                let a = p[3];
                if a > 0.0 {
                    (0..3).for_each(|c| acc[c] += p[c] * a);
                    acc[3] += a;
                }
            }
        }
        let n = (taps * taps) as f32;
        if acc[3] <= 0.0 {
            Rgba([0.0; 4])
        } else {
            Rgba([
                acc[0] / acc[3],
                acc[1] / acc[3],
                acc[2] / acc[3],
                acc[3] / n,
            ])
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 300x100 bar, low in a 512 frame, must come out centred, 72 % of the plate wide, and
    /// inside the safe square.
    #[test]
    fn fits_into_the_safe_square() {
        let alpha = Plane::from_fn(512, 512, |x, y| {
            if (100..400).contains(&x) && (350..450).contains(&y) {
                1.0
            } else {
                0.0
            }
        });
        let rgb = Rgba32FImage::from_fn(512, 512, |x, y| {
            Rgba([1.0, 0.0, 0.0, alpha.at(i64::from(x), i64::from(y))])
        });
        let t = Template::default();
        let out = fit_object(&rgb, &alpha, 1024, &t).expect("object");
        let a = Plane::from_fn(1024, 1024, |x, y| out.get_pixel(x, y).0[3]);
        let b = alpha_bbox(&a, 0.1).expect("bbox");
        let grid = PlateGrid::for_canvas(1024, &t);
        let want = t.fill.0 * grid.side as f32;
        assert!(
            (b.width() - want).abs() <= 2.0,
            "width {} want {want}",
            b.width()
        );
        let (cx, _) = centroid(&a);
        assert!((cx - 512.0).abs() < 2.0, "cx {cx}");
        let safe_lo = 512.0 - t.safe.0 * grid.side as f32 / 2.0;
        assert!(
            b.x0 >= safe_lo - 1.0
                && b.y0 >= safe_lo - 1.0
                && b.x1 <= 1024.0 - safe_lo + 1.0
                && b.y1 <= 1024.0 - safe_lo + 1.0,
            "{b:?}"
        );
    }

    #[test]
    fn empty_alpha_is_an_error() {
        let alpha = Plane::filled(8, 8, 0.0);
        let rgb = Rgba32FImage::new(8, 8);
        assert!(matches!(
            fit_object(&rgb, &alpha, 64, &Template::default()),
            Err(IconsError::EmptyObject)
        ));
    }

    const SHIFTS: &[(&str, f32, f32, f32, f32, f32, f32)] = &[
        ("inside stays", 10.0, 0.0, 10.0, 0.0, 100.0, 10.0),
        ("pushed right", -5.0, 0.0, 10.0, 0.0, 100.0, 0.0),
        ("pushed left", 95.0, 0.0, 10.0, 0.0, 100.0, 90.0),
    ];

    #[test]
    fn clamp_shift_cases() {
        for (name, t, lo, hi, min, max, want) in SHIFTS {
            assert_eq!(clamp_shift(*t, *lo, *hi, *min, *max), *want, "{name}");
        }
    }
}
