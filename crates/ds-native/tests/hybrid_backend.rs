//! The harness's vello_hybrid backend (sill Q330): the same document painted on the GPU reads
//! back as vello_cpu paints it, and the on-demand benchmark times a 300-cell COLRv1 emoji grid
//! scrolling for 180 frames on both backends (FINDINGS "Hybrid harness backend"). Every GPU case
//! skips, with a note, where no adapter opens (CI).
//!
//! `cargo test -p ds-native --release --test hybrid_backend -- --ignored --nocapture`

use dioxus::prelude::*;
use ds::{Appearance, Ds, Material, Point, Px};
use ds_native::{AdapterPref, Backend, Harness, HarnessConfig, Viewport};
use std::time::{Duration, Instant};

const COLRV1: &str = "/usr/share/fonts/google-noto-color-emoji-fonts/Noto-COLRv1.ttf";

const SMALL: Viewport = Viewport {
    width: 240,
    height: 120,
    scale_percent: 100,
};

fn swatches() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "position:absolute; left:10px; top:10px; width:100px; height:40px; background:#e03c31; border-radius:8px;" }
            div { style: "position:absolute; left:120px; top:10px; width:100px; height:40px; background:linear-gradient(90deg,#1060ff,#20c060);" }
            div { style: "position:absolute; left:10px; top:60px; width:220px; height:50px; font-size:20px; color:#111;", "Hybrid harness" }
        }
    }
}

/// A hybrid harness of `app` at `viewport`, or `None` (with a note) where no GPU opens.
fn hybrid(app: fn() -> Element, viewport: Viewport, pref: AdapterPref) -> Option<Harness> {
    let config = HarnessConfig::new(viewport)
        .with_backend(Backend::Hybrid)
        .with_adapter(pref);
    match Harness::try_with_config(app, config) {
        Ok(harness) => Some(harness),
        Err(error) => {
            eprintln!("skipped: no GPU ({error})");
            None
        }
    }
}

#[test]
fn hybrid_pictures_read_back_as_vello_cpu_paints_them() {
    let Some(mut gpu) = hybrid(swatches, SMALL, AdapterPref::Auto) else {
        return;
    };
    let mut cpu = Harness::new(swatches, SMALL);
    let (a, b) = (
        cpu.render().expect("cpu paints"),
        gpu.render().expect("gpu paints"),
    );
    assert_eq!(a.dimensions(), b.dimensions());
    // The same scene, rasterised twice: anti-aliased edges may differ by a few levels.
    let far = a
        .pixels()
        .zip(b.pixels())
        .filter(|(p, q)| p.0.iter().zip(q.0).any(|(x, y)| x.abs_diff(y) > 24))
        .count();
    let total = (a.width() * a.height()) as usize;
    assert!(far * 100 < total, "{far} of {total} pixels differ");
    // The red swatch's middle, exactly.
    assert_eq!(b.get_pixel(60, 30).0, a.get_pixel(60, 30).0);
    assert!(gpu.adapter().is_some());
    assert!(cpu.adapter().is_none());
}

#[test]
fn a_hybrid_frame_times_the_gpu_finishing_it() {
    let Some(mut gpu) = hybrid(swatches, SMALL, AdapterPref::Auto) else {
        return;
    };
    let time = gpu.paint_timed().expect("paints");
    assert!(time.total >= time.scene);
    assert_eq!(time.render(), time.total - time.scene);
}

// The benchmark: the emoji picker's worst case, 300 COLRv1 cells in one scroll container.

const GRID: Viewport = Viewport {
    width: 400,
    height: 480,
    scale_percent: 100,
};
const CELLS: u32 = 300;
const FRAMES: usize = 180;
/// Pixels per frame: 180 frames scroll 1,260 of the grid's 1,344 px of travel.
const STEP: f32 = 7.0;

/// 300 consecutive pictographs from U+1F300, every one a distinct COLRv1 glyph.
fn emoji(count: u32) -> Vec<String> {
    (0..count)
        .filter_map(|i| char::from_u32(0x1F300 + i))
        .map(String::from)
        .collect()
}

/// The picker's grid: 50 x 48 px cells, eight to a row, in one scroll container.
fn grid(cells: Vec<String>, class: &'static str) -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { id: "grid", style: "width:400px; height:480px; overflow-y:scroll; display:flex; flex-wrap:wrap; align-content:flex-start; background:#fff;",
                for (i, label) in cells.into_iter().enumerate() {
                    span { key: "{i}", class, style: "display:block; width:50px; height:48px; font-size:32px; line-height:48px; text-align:center; color:#222;", "{label}" }
                }
            }
        }
    }
}

/// A benchmarked grid: its name and its app.
type Grid = (&'static str, fn() -> Element);

/// The benchmark proper: 300 `.ds-emoji-text` cells.
fn emoji_grid() -> Element {
    grid(emoji(CELLS), "ds-emoji-text")
}

/// Control: the same 300 cells holding two Inter letters, to price the COLRv1 glyphs.
fn text_grid() -> Element {
    grid(vec!["Ab".to_owned(); CELLS as usize], "")
}

/// Control: the first 104 cells only (13 rows, one past the viewport), to show whether cells
/// scrolled out of view cost anything.
fn short_grid() -> Element {
    grid(emoji(104), "ds-emoji-text")
}

/// Median, 95th percentile and maximum of `times`.
fn spread(mut times: Vec<Duration>) -> [Duration; 3] {
    times.sort();
    let at = |q: usize| times[(times.len() * q / 100).min(times.len() - 1)];
    [at(50), at(95), times[times.len() - 1]]
}

fn ms(time: Duration) -> String {
    format!("{:.2}", time.as_secs_f64() * 1000.0)
}

/// Scroll `harness`'s grid for `FRAMES` frames, painting each, and print the spreads.
fn scroll(label: &str, mut harness: Harness) {
    let first = harness.paint_timed().expect("paints");
    let at = Point {
        x: Px(200.0),
        y: Px(240.0),
    };
    let (mut frames, mut scenes, mut layouts) = (Vec::new(), Vec::new(), Vec::new());
    // Blitz's client rects ignore an element's own scroll offset, so the scroll is proved by
    // the pixels.
    let before = harness.render().expect("paints");
    for _ in 0..FRAMES {
        let started = Instant::now();
        harness.wheel(at, Px(0.0), Px(-STEP));
        layouts.push(started.elapsed());
        let time = harness.paint_timed().expect("paints");
        frames.push(time.total);
        scenes.push(time.scene);
    }
    let after = harness.render().expect("paints");
    if let Ok(dir) = std::env::var("EMOJI_OUT") {
        let slug: String = label.chars().filter(char::is_ascii_alphanumeric).collect();
        let _ = before.save(format!("{dir}/emoji_grid_{slug}_top.png"));
        let _ = after.save(format!("{dir}/emoji_grid_{slug}.png"));
    }
    assert!(before != after, "the grid did not scroll");
    let row = |name: &str, times: Vec<Duration>| {
        let [p50, p95, max] = spread(times);
        println!(
            "  {name:<22} p50 {:>6}  p95 {:>6}  max {:>6} ms",
            ms(p50),
            ms(p95),
            ms(max)
        );
    };
    println!(
        "{label}: first frame {} ms (scene {} ms)",
        ms(first.total),
        ms(first.scene)
    );
    row("paint (frame)", frames.clone());
    row("  of which scene", scenes);
    row("wheel + style/layout", layouts);
}

#[test]
#[ignore = "a benchmark: run with --ignored --nocapture, in release"]
fn emoji_grid_scrolls_on_both_backends() {
    if !std::path::Path::new(COLRV1).exists() {
        eprintln!("skipped: {COLRV1} is not installed");
        return;
    }
    println!(
        "{CELLS} COLRv1 cells, {}x{} @ {}%, {FRAMES} frames of {STEP} px",
        GRID.width, GRID.height, GRID.scale_percent
    );
    let grids: [Grid; 3] = [
        ("300 emoji", emoji_grid),
        ("300 text cells", text_grid),
        ("104 emoji", short_grid),
    ];
    for (name, app) in grids {
        println!("== {name}");
        scroll("vello_cpu", Harness::new(app, GRID));
        let mut seen = Vec::new();
        for pref in [AdapterPref::Discrete, AdapterPref::Integrated] {
            let Some(harness) = hybrid(app, GRID, pref) else {
                continue;
            };
            let adapter = harness.adapter().unwrap_or_default().to_owned();
            if seen.contains(&adapter) {
                continue;
            }
            seen.push(adapter.clone());
            scroll(&format!("vello_hybrid on {adapter}"), harness);
        }
    }
}
