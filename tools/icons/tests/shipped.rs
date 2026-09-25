//! The consistency pass over the shipped set (design/08-ICONS.md 2.11): the five exported 512 px
//! icons, three procedural and two Klein-derived, must share the plate's silhouette, the bevel
//! arc, and the drop shadow's reach, whatever drew their face.

use std::path::PathBuf;

use icons::{PlateGrid, Template, oklab, plate_mask};
use image::Rgba32FImage;

const APPS: [&str; 5] = ["mail", "files", "terminal", "notes", "photos"];

fn load(app: &str, style: &str) -> Rgba32FImage {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/icons/apps");
    let path = match style {
        "" => root.join(app).join("512.png"),
        s => root.join(app).join(s).join("512.png"),
    };
    image::open(&path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
        .to_rgba32f()
}

fn lightness(img: &Rgba32FImage, x: u32, y: u32) -> f32 {
    let p = img.get_pixel(x, y).0;
    oklab([p[0], p[1], p[2]]).l
}

/// Pixels inside the plate square whose opacity disagrees with the squircle mask by more than
/// half coverage.
fn silhouette_misses(img: &Rgba32FImage, grid: PlateGrid, t: &Template) -> usize {
    let mask = plate_mask(grid, t);
    let range = grid.origin..grid.origin + grid.side;
    range
        .clone()
        .flat_map(|y| range.clone().map(move |x| (x, y)))
        .filter(|&(x, y)| {
            let m = mask.at(i64::from(x), i64::from(y));
            // Inside the plate the icon is opaque; outside it only the soft shadow remains.
            let a = img.get_pixel(x, y).0[3];
            (m > 0.99 && a < 0.99) || (m < 0.01 && a > 0.6)
        })
        .count()
}

/// How much lighter the plate's top edge is than the plate just below it, at the centre column:
/// the bevel arc's strength.
fn arc(img: &Rgba32FImage, grid: PlateGrid) -> f32 {
    let c = grid.origin + grid.side / 2;
    let top = (grid.origin + 1..grid.origin + 6)
        .map(|y| lightness(img, c, y))
        .fold(f32::MIN, f32::max);
    top - lightness(img, c, grid.origin + 24)
}

/// How much darker the plate's bottom edge is than the plate just above it: the bevel's shade.
fn shade(img: &Rgba32FImage, grid: PlateGrid) -> f32 {
    let c = grid.origin + grid.side / 2;
    let bottom = grid.origin + grid.side;
    let low = (bottom - 6..bottom - 1)
        .map(|y| lightness(img, c, y))
        .fold(f32::MAX, f32::min);
    lightness(img, c, bottom - 24) - low
}

/// The plate's own lightness, sampled left of centre at mid height (clear of any symbol).
fn plate_l(img: &Rgba32FImage, grid: PlateGrid) -> f32 {
    lightness(img, grid.origin + 12, grid.origin + grid.side / 2)
}

/// A plate this light has no room for the white arc; its bevel shows as the shade and rim.
const NEAR_WHITE: f32 = 0.93;

/// The last row under the plate, at the centre column, where the drop shadow is still visible.
fn shadow_reach(img: &Rgba32FImage, grid: PlateGrid) -> u32 {
    let c = grid.origin + grid.side / 2;
    (grid.origin + grid.side..img.height())
        .take_while(|&y| img.get_pixel(c, y).0[3] > 0.02)
        .last()
        .unwrap_or(0)
}

#[test]
fn the_five_share_one_plate() {
    let t = Template::default();
    let grid = PlateGrid::for_canvas(512, &t);
    for style in ["", "muted", "monochrome"] {
        let imgs: Vec<_> = APPS.iter().map(|a| load(a, style)).collect();
        for (app, img) in APPS.iter().zip(&imgs) {
            assert_eq!(img.dimensions(), (512, 512), "{app} {style}");
            let misses = silhouette_misses(img, grid, &t);
            assert!(
                misses < 40,
                "{app} {style}: {misses} pixels off the squircle"
            );
        }
        for (app, img) in APPS.iter().zip(&imgs) {
            let s = shade(img, grid);
            assert!(s > 0.01, "{app} {style}: no bevel shade ({s})");
        }
        let arcs: Vec<f32> = imgs
            .iter()
            .filter(|i| plate_l(i, grid) < NEAR_WHITE)
            .map(|i| arc(i, grid))
            .collect();
        assert!(arcs.len() >= 4, "{style}: at most one near-white plate");
        for a in &arcs {
            assert!(*a > 0.015, "{style}: no bevel arc ({a}) in {arcs:?}");
        }
        let spread = arcs.iter().fold(f32::MIN, |m, a| m.max(*a))
            - arcs.iter().fold(f32::MAX, |m, a| m.min(*a));
        assert!(spread < 0.08, "{style}: bevel arcs differ: {arcs:?}");
        let reach: Vec<u32> = imgs.iter().map(|i| shadow_reach(i, grid)).collect();
        let (lo, hi) = (reach.iter().min(), reach.iter().max());
        assert!(
            hi.zip(lo).is_some_and(|(h, l)| h - l <= 2),
            "{style}: shadow reach differs: {reach:?}"
        );
    }
}

/// Every app ships every size in every style.
#[test]
fn every_size_of_every_style_is_there() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/icons/apps");
    for app in APPS {
        for style in ["", "muted", "monochrome"] {
            for px in icons::SHIP_PX {
                let p = root.join(app).join(style).join(format!("{px}.png"));
                let (w, h) =
                    image::image_dimensions(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
                assert_eq!((w, h), (px, px), "{}", p.display());
            }
        }
    }
}

/// The Monochrome set is exported neutral: no pixel carries more than a trace of colour, so the
/// run-time tint is the only hue it gets.
#[test]
fn monochrome_is_exported_neutral() {
    for app in APPS {
        let img = load(app, "monochrome");
        let max = img
            .pixels()
            .filter(|p| p.0[3] > 0.5)
            .map(|p| {
                let o = oklab([p.0[0], p.0[1], p.0[2]]);
                o.a.hypot(o.b)
            })
            .fold(0.0_f32, f32::max);
        assert!(max < 0.02, "{app}: chroma {max}");
    }
}
