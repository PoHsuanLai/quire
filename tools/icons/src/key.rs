use std::collections::VecDeque;

use image::{Rgba, Rgba32FImage};

use crate::{Oklab, Plane, Template, compose::over, delta_e, oklab};

/// The keyed render: the object's own alpha (fit uses it) and the full keyed layer, which
/// also carries the model's contact shadow as a translucent dark matte.
#[derive(Debug, Clone, PartialEq)]
pub struct Key {
    /// Object coverage only, 0..=1.
    pub alpha: Plane,
    /// The object over its shadow matte, straight alpha, same size as the render.
    pub layer: Rgba32FImage,
    /// Corner samples, top-left, top-right, bottom-left, bottom-right.
    pub corners: [Oklab; 4],
}

/// Colour of the shadow matte (the `--shadow-2` ink, 08 2.5).
const SHADOW_INK: [f32; 3] = [
    0x1A as f32 / 255.0,
    0x1E as f32 / 255.0,
    0x1A as f32 / 255.0,
];

/// Keys a flat studio background out to alpha (design/08-ICONS.md 3.7 step 1).
///
/// The background is modelled as the bilinear blend of the four corner samples (renders have a
/// soft vignette). It is grown from the image border: a pixel joins when it looks like ground
/// or shadow on it (chroma within `key_chroma` of the local background, at most `key_lighter`
/// lighter and `key_darker` darker) AND is within `key_step` ΔE_OK of the neighbour it is
/// reached from, so smooth vignettes and soft shadows join while any drawn edge stops the
/// growth, even round a white face on a light ground. Everything not reached is object. A
/// reached pixel darker than its local background becomes shadow matte with alpha
/// `1 - Y / Y_background` (so it darkens the plate the way it darkened the ground).
pub fn key_background(img: &Rgba32FImage, t: &Template) -> Key {
    let (w, h) = img.dimensions();
    let corners = corner_samples(img);
    let lab: Vec<Oklab> = img
        .pixels()
        .map(|p| oklab([p.0[0], p.0[1], p.0[2]]))
        .collect();
    let local = |i: usize| {
        bilinear(
            &corners,
            (i as u32 % w) as f32 / (w - 1).max(1) as f32,
            (i as u32 / w) as f32 / (h - 1).max(1) as f32,
        )
    };
    let open = |i: usize| {
        let (p, g) = (lab[i], local(i));
        let chroma = ((p.a - g.a).powi(2) + (p.b - g.b).powi(2)).sqrt();
        chroma < t.key_chroma && p.l - g.l < t.key_lighter && g.l - p.l < t.key_darker
    };
    let reached = grow_from_border(w as usize, h as usize, open, |i, j| {
        delta_e(lab[i], lab[j]) < t.key_step
    });
    let object = drop_specks(
        w as usize,
        h as usize,
        &reached.iter().map(|r| !r).collect::<Vec<_>>(),
        t.key_speck,
    );
    let hard = Plane {
        width: w,
        height: h,
        data: object.iter().map(|&o| if o { 1.0 } else { 0.0 }).collect(),
    };
    let alpha = hard.blurred(t.key_feather);
    let shadow = Plane::from_fn(w, h, |x, y| {
        let i = (y * w + x) as usize;
        let ratio = (lab[i].l / local(i).l.max(1e-3)).powi(3);
        ((1.0 - ratio - t.shadow_floor) / (1.0 - t.shadow_floor)).clamp(0.0, t.shadow_max)
    });
    let layer = Rgba32FImage::from_fn(w, h, |x, y| {
        let (a, s) = (
            alpha.at(i64::from(x), i64::from(y)),
            shadow.at(i64::from(x), i64::from(y)),
        );
        let p = img.get_pixel(x, y).0;
        Rgba(over(
            [p[0], p[1], p[2], a],
            [SHADOW_INK[0], SHADOW_INK[1], SHADOW_INK[2], s * (1.0 - a)],
        ))
    });
    Key {
        alpha,
        layer,
        corners,
    }
}

fn corner_samples(img: &Rgba32FImage) -> [Oklab; 4] {
    let (w, h) = img.dimensions();
    let n = (w.min(h) / 32).max(1);
    let patch = |x0: u32, y0: u32| {
        let px: Vec<[f32; 4]> = (y0..y0 + n)
            .flat_map(|y| (x0..x0 + n).map(move |x| (x, y)))
            .map(|(x, y)| img.get_pixel(x, y).0)
            .collect();
        let mean = |c: usize| px.iter().map(|p| p[c]).sum::<f32>() / px.len() as f32;
        oklab([mean(0), mean(1), mean(2)])
    };
    [
        patch(0, 0),
        patch(w - n, 0),
        patch(0, h - n),
        patch(w - n, h - n),
    ]
}

fn bilinear(c: &[Oklab; 4], u: f32, v: f32) -> Oklab {
    let mix = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let lerp = |p: Oklab, q: Oklab, t: f32| Oklab {
        l: mix(p.l, q.l, t),
        a: mix(p.a, q.a, t),
        b: mix(p.b, q.b, t),
    };
    lerp(lerp(c[0], c[1], u), lerp(c[2], c[3], u), v)
}

/// Keeps the 4-connected components of `on` whose area is at least `fraction` of the largest:
/// unreached specks of ground are not object, while a detached part of a real object is.
fn drop_specks(w: usize, h: usize, on: &[bool], fraction: f32) -> Vec<bool> {
    let mut label = vec![usize::MAX; w * h];
    let mut areas = Vec::new();
    for start in 0..w * h {
        if !on[start] || label[start] != usize::MAX {
            continue;
        }
        let id = areas.len();
        let mut stack = vec![start];
        label[start] = id;
        let mut area = 0usize;
        while let Some(i) = stack.pop() {
            area += 1;
            let (x, y) = (i % w, i / w);
            let next = [
                (x > 0).then(|| i - 1),
                (x + 1 < w).then(|| i + 1),
                (y > 0).then(|| i - w),
                (y + 1 < h).then(|| i + w),
            ];
            for j in next.into_iter().flatten() {
                if on[j] && label[j] == usize::MAX {
                    label[j] = id;
                    stack.push(j);
                }
            }
        }
        areas.push(area);
    }
    let min = areas.iter().max().map_or(0.0, |&m| m as f32 * fraction);
    label
        .iter()
        .map(|&l| l != usize::MAX && areas[l] as f32 >= min)
        .collect()
}

/// Region growing from the border (4-connected): a pixel joins when `open(j)` holds and the
/// step from its neighbour `i` passes `step(i, j)`.
fn grow_from_border(
    w: usize,
    h: usize,
    open: impl Fn(usize) -> bool,
    step: impl Fn(usize, usize) -> bool,
) -> Vec<bool> {
    let mut seen = vec![false; w * h];
    let border = (0..w)
        .flat_map(|x| [x, (h - 1) * w + x])
        .chain((0..h).flat_map(|y| [y * w, y * w + w - 1]));
    let mut queue: VecDeque<usize> = border.filter(|&i| open(i)).collect();
    queue.iter().for_each(|&i| seen[i] = true);
    while let Some(i) = queue.pop_front() {
        let (x, y) = (i % w, i / w);
        let next = [
            (x > 0).then(|| i - 1),
            (x + 1 < w).then(|| i + 1),
            (y > 0).then(|| i - w),
            (y + 1 < h).then(|| i + w),
        ];
        for j in next.into_iter().flatten() {
            if !seen[j] && open(j) && step(i, j) {
                seen[j] = true;
                queue.push_back(j);
            }
        }
    }
    seen
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Grey ground, a red ring with a grey hole: the ground keys out, the ring and hole stay.
    fn ring() -> Rgba32FImage {
        Rgba32FImage::from_fn(64, 64, |x, y| {
            let r = ((x as f32 - 32.0).powi(2) + (y as f32 - 32.0).powi(2)).sqrt();
            if (8.0..16.0).contains(&r) {
                Rgba([0.9, 0.2, 0.2, 1.0])
            } else {
                Rgba([0.85, 0.85, 0.86, 1.0])
            }
        })
    }

    const CASES: &[(&str, u32, u32, f32)] = &[
        ("corner", 0, 0, 0.0),
        ("ground beside ring", 60, 32, 0.0),
        ("ring", 32 + 12, 32, 1.0),
        ("enclosed hole", 32, 32, 1.0),
    ];

    /// A white disc on a light vignetted ground with a soft dark blob beside it: the disc
    /// stays whole although it is close to the ground's colour, the blob becomes shadow.
    fn white_on_light() -> Rgba32FImage {
        Rgba32FImage::from_fn(128, 128, |x, y| {
            let vignette = 0.84
                + 0.04
                    * (1.0 - ((x as f32 - 64.0).powi(2) + (y as f32 - 64.0).powi(2)).sqrt() / 90.0);
            let r = ((x as f32 - 50.0).powi(2) + (y as f32 - 50.0).powi(2)).sqrt();
            let blob = ((x as f32 - 95.0).powi(2) + (y as f32 - 95.0).powi(2)).sqrt();
            if r < 25.0 {
                Rgba([0.97, 0.96, 0.94, 1.0])
            } else {
                let dark = 0.25 * (-(blob / 20.0).powi(2)).exp();
                let v = vignette * (1.0 - dark);
                Rgba([v, v, v * 1.01, 1.0])
            }
        })
    }

    #[test]
    fn white_face_stays_and_shadow_is_matte() {
        let key = key_background(&white_on_light(), &Template::default());
        assert!(
            key.alpha.at(50, 50) > 0.99,
            "disc centre {}",
            key.alpha.at(50, 50)
        );
        assert!(
            key.alpha.at(50, 30) > 0.99,
            "disc near its edge {}",
            key.alpha.at(50, 30)
        );
        assert!(
            key.alpha.at(95, 95) < 0.01,
            "blob is not object {}",
            key.alpha.at(95, 95)
        );
        let blob = key.layer.get_pixel(95, 95).0;
        assert!(
            blob[3] > 0.2 && blob[0] < 0.2,
            "blob is dark matte {blob:?}"
        );
        assert!(key.layer.get_pixel(2, 2).0[3] < 0.01, "ground is clear");
    }

    #[test]
    fn specks_go_parts_stay() {
        // 10x10 grid: a 5x5 block, a 2x2 part (16 % of it, above the 10 % cut), one lone pixel.
        let on: Vec<bool> = (0..100)
            .map(|i| {
                let (x, y) = (i % 10, i / 10);
                (x < 5 && y < 5)
                    || ((7..9).contains(&x) && (7..9).contains(&y))
                    || (x == 9 && y == 0)
            })
            .collect();
        let kept = drop_specks(10, 10, &on, 0.1);
        assert!(kept[0] && kept[88], "block and part stay");
        assert!(!kept[9], "lone pixel goes");
    }

    #[test]
    fn keys_only_connected_ground() {
        let key = key_background(&ring(), &Template::default());
        for (name, x, y, want) in CASES {
            let got = key.alpha.at(i64::from(*x), i64::from(*y));
            assert!(
                (got - want).abs() < 0.05,
                "{name}: alpha {got}, want {want}"
            );
        }
    }
}
