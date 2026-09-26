//! Colour emoji on the headless path (FINDINGS "Colour emoji"): with `'Noto Color Emoji'` named
//! in the stack, the installed COLRv1 build paints 😀 in colour through Blitz, parley, skrifa and
//! vello_cpu, and a flag (two regional indicators) shapes to one coloured glyph. The system's
//! own fallback does not reach it on this machine (it picks Symbola, monochrome), which is why
//! an emoji grid names the family. Skipped, with a note, where the font is not installed.

use dioxus::prelude::*;
use ds::{Appearance, Ds, Material};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const COLRV1: &str = "/usr/share/fonts/google-noto-color-emoji-fonts/Noto-COLRv1.ttf";

const VIEW: Viewport = Viewport {
    width: 200,
    height: 80,
    scale_percent: 100,
};

fn app() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "position:absolute; left:0; top:0; width:200px; height:80px; background:#fff; display:flex; font-family:'Inter', 'Noto Color Emoji', sans-serif; font-size:48px; line-height:72px;",
                span { id: "face", style: "display:block; width:88px;", "😀" }
                span { id: "flag", style: "display:block; width:88px;", "🇹🇼" }
            }
        }
    }
}

/// Coloured pixels (channel spread over 60) inside `selector`'s box.
fn coloured(harness: &Harness, shot: &image::RgbaImage, selector: &str) -> u32 {
    let rect = harness.rect(selector).expect("laid out");
    let xs = rect.left().0 as u32..(rect.left().0 + rect.size.width.0) as u32;
    let ys = rect.top().0 as u32..(rect.top().0 + rect.size.height.0) as u32;
    ys.flat_map(|y| xs.clone().map(move |x| (x, y)))
        .filter(|(x, y)| x < &shot.width() && y < &shot.height())
        .filter(|(x, y)| {
            let [r, g, b, _] = shot.get_pixel(*x, *y).0;
            r.max(g).max(b) - r.min(g).min(b) > 60
        })
        .count() as u32
}

#[test]
fn colrv1_emoji_paint_in_colour_on_vello_cpu() {
    if !std::path::Path::new(COLRV1).exists() {
        eprintln!("skipped: {COLRV1} is not installed");
        return;
    }
    let mut harness = Harness::new(app, VIEW);
    harness.advance(Duration::from_millis(100));
    let shot = harness.render().expect("renders");
    let face = coloured(&harness, &shot, "#face");
    let flag = coloured(&harness, &shot, "#flag");
    // A 48 px face covers about 2,000 pixels, 1,700 of them coloured; a monochrome face none.
    assert!(face > 1000, "😀 coloured pixels: {face}");
    assert!(flag > 1000, "🇹🇼 coloured pixels: {flag}");
}
