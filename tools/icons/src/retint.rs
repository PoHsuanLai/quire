//! Brings a model-drawn face into a dialect (design/08-ICONS.md 2.10): the Klein round-three
//! picks are soft clay emboss on an uneven two-hue ground; retinting keeps their relief (the
//! lightness detail) and replaces their colour with the dialect's.
//!
//! The ground is modelled as a plane in lightness, fitted to a ring of border pixels (where the
//! symbol never is); what differs from that plane is the symbol and its emboss. The plate takes
//! the dialect's plate lightness, the relief is added on top with a gain, and hue and chroma are
//! set from the dialect's roles, interpolated by lightness from plate to symbol.

use image::{Rgba, Rgba32FImage};

use crate::{Oklab, Roles, oklab, srgb};

/// The relief is scaled so this percentile of the pixels standing above the ground lands on the
/// symbol's lightness: Klein's pale symbols sit only 0.05-0.15 L above their ground, and by
/// different amounts per render, too little at 16 px.
const RELIEF_PERCENTILE: f32 = 0.9;
/// Bounds on that gain.
const GAIN_MIN: f32 = 1.0;
const GAIN_MAX: f32 = 5.0;
/// Ground mottling below this lightness difference is shrunk towards the plate before the gain,
/// so the gain lifts the symbol, not the model's texture.
const QUIET: f32 = 0.02;
/// The border ring used to fit the ground, as a share of the side.
const RING: f32 = 0.06;

fn lightness(img: &Rgba32FImage, x: u32, y: u32) -> f32 {
    let p = img.get_pixel(x, y).0;
    oklab([p[0], p[1], p[2]]).l
}

/// Least-squares plane `l = a + b x + c y` through the ring's pixels (x, y in 0..=1).
fn ground_plane(img: &Rgba32FImage) -> [f32; 3] {
    let (w, h) = (img.width(), img.height());
    let ring = (RING * w.min(h) as f32).max(1.0) as u32;
    let samples = (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .filter(|&(x, y)| x < ring || y < ring || x >= w - ring || y >= h - ring)
        .step_by(3)
        .map(|(x, y)| {
            (
                f64::from(x) / f64::from(w),
                f64::from(y) / f64::from(h),
                f64::from(lightness(img, x, y)),
            )
        });
    // Normal equations for [1, x, y].
    let mut m = [[0.0f64; 3]; 3];
    let mut v = [0.0f64; 3];
    for (x, y, l) in samples {
        let row = [1.0, x, y];
        for i in 0..3 {
            v[i] += row[i] * l;
            for j in 0..3 {
                m[i][j] += row[i] * row[j];
            }
        }
    }
    solve3(m, v).map(|c| c as f32)
}

/// Cramer's rule on a 3x3 system; a singular system gives the flat mean.
fn solve3(m: [[f64; 3]; 3], v: [f64; 3]) -> [f64; 3] {
    let det = |a: [[f64; 3]; 3]| {
        a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1])
            - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0])
            + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0])
    };
    let d = det(m);
    if d.abs() < 1e-12 {
        return [v[0] / m[0][0].max(1.0), 0.0, 0.0];
    }
    let col = |k: usize| {
        let mut a = m;
        for (row, value) in a.iter_mut().zip(v) {
            row[k] = value;
        }
        det(a) / d
    };
    [col(0), col(1), col(2)]
}

/// `face` re-coloured into `roles`: the ground becomes the plate colour, what stands above it
/// moves towards the symbol colour, what sinks below it towards a darker plate.
pub fn retint(face: &Rgba32FImage, roles: &Roles) -> Rgba32FImage {
    let [a, b, c] = ground_plane(face);
    let (w, h) = (face.width(), face.height());
    let (plate, symbol) = (roles.plate, roles.symbol);
    let relief_at = |x: u32, y: u32| {
        let r = lightness(face, x, y) - (a + b * (x as f32 / w as f32) + c * (y as f32 / h as f32));
        r.signum() * (r.abs() - QUIET * (1.0 - (r.abs() / (2.0 * QUIET)).min(1.0))).max(0.0)
    };
    let mut raised: Vec<f32> = (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .step_by(7)
        .map(|(x, y)| relief_at(x, y))
        .filter(|r| *r > 0.02)
        .collect();
    raised.sort_by(f32::total_cmp);
    let top = raised
        .get(((raised.len() as f32 - 1.0) * RELIEF_PERCENTILE) as usize)
        .copied()
        .unwrap_or(0.1);
    let gain = ((symbol.l - plate.l).abs() / top.max(1e-3)).clamp(GAIN_MIN, GAIN_MAX);
    Rgba32FImage::from_fn(w, h, |x, y| {
        let relief = relief_at(x, y) * gain;
        let l = (plate.l + relief * (symbol.l - plate.l).signum()).clamp(0.0, 1.0);
        let t = ((l - plate.l) / (symbol.l - plate.l)).clamp(0.0, 1.0);
        let [r, g, bl] = srgb(Oklab {
            l,
            a: plate.a + (symbol.a - plate.a) * t,
            b: plate.b + (symbol.b - plate.b) * t,
        });
        Rgba([r, g, bl, face.get_pixel(x, y).0[3]])
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Dialect, roles};

    /// A ground with a strong left-to-right lightness ramp and a lighter square in the middle:
    /// after retinting, the ground is flat plate colour and the square is lighter than it.
    #[test]
    fn ramp_is_flattened_and_the_symbol_kept() {
        let img = Rgba32FImage::from_fn(128, 128, |x, y| {
            let ramp = 0.45 + 0.3 * x as f32 / 128.0;
            let v = match (40..88).contains(&x) && (40..88).contains(&y) {
                true => ramp + 0.15,
                false => ramp,
            };
            Rgba([v, v, v, 1.0])
        });
        let r = roles(Dialect::Monochrome, "sage".parse().expect("sage"));
        let out = retint(&img, &r);
        let l = |x, y| lightness(&out, x, y);
        assert!(
            (l(4, 64) - l(123, 64)).abs() < 0.02,
            "ramp gone: {} {}",
            l(4, 64),
            l(123, 64)
        );
        assert!(
            (l(4, 64) - r.plate.l).abs() < 0.03,
            "ground at plate lightness"
        );
        assert!(l(64, 64) > l(4, 64) + 0.15, "symbol stands out");
        let p = out.get_pixel(4, 64).0;
        let o = oklab([p[0], p[1], p[2]]);
        assert!(
            (o.b.atan2(o.a).to_degrees().rem_euclid(360.0) - 150.0).abs() < 5.0,
            "sage hue"
        );
    }
}
