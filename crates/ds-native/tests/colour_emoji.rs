//! Colour emoji on the headless path (FINDINGS "Colour emoji"): with `'Noto Color Emoji'` named
//! in the stack, the installed COLRv1 build paints 😀 in colour through Blitz, parley, skrifa and
//! vello_cpu, and a flag (two regional indicators) shapes to one coloured glyph. The system's
//! own fallback does not reach it on this machine (it picks Symbola, monochrome), which is why
//! an emoji grid names the family. `.ds-emoji-text` (the `--font-emoji` stack, emoji face first)
//! paints colour too, while its letters, digits and `#` stay Inter's pixel for pixel. Skipped,
//! with a note, where the font is not installed.

use dioxus::prelude::*;
use ds::{Appearance, Ds, Material};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const COLRV1: &str = "/usr/share/fonts/google-noto-color-emoji-fonts/Noto-COLRv1.ttf";

const VIEW: Viewport = Viewport {
    width: 400,
    height: 160,
    scale_percent: 100,
};

fn app() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "position:absolute; left:0; top:0; width:200px; height:80px; background:#fff; display:flex; font-family:'Inter', 'Noto Color Emoji', sans-serif; font-size:48px; line-height:72px;",
                span { id: "face", style: "display:block; width:88px;", "😀" }
                span { id: "flag", style: "display:block; width:88px;", "🇹🇼" }
            }
            div { style: "position:absolute; left:0; top:80px; width:400px; height:80px; background:#fff; display:flex; font-size:48px; line-height:72px; color:#000;",
                span { id: "token", class: "ds-emoji-text", style: "display:block; width:88px;", "🎉" }
                span { id: "token-text", class: "ds-emoji-text", style: "display:block; width:150px;", "Ab 2#" }
                span { id: "inter-text", style: "display:block; width:150px; font-family:'Inter';", "Ab 2#" }
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
    // `.ds-emoji-text` (`--font-emoji`) paints an emoji in colour with no family named by the caller,
    let token = coloured(&harness, &shot, "#token");
    assert!(
        token > 1000,
        "🎉 under .ds-emoji-text coloured pixels: {token}"
    );
    // and its letters, digits and `#` fall through to Inter: the same pixels as Inter alone.
    let crop = |selector: &str| {
        let rect = harness.rect(selector).expect("laid out");
        image::imageops::crop_imm(
            &shot,
            rect.left().0 as u32,
            rect.top().0 as u32,
            rect.size.width.0 as u32,
            rect.size.height.0 as u32,
        )
        .to_image()
    };
    let (ours, inter) = (crop("#token-text"), crop("#inter-text"));
    if let Ok(dir) = std::env::var("EMOJI_OUT") {
        let _ = shot.save(format!("{dir}/colour_emoji.png"));
    }
    assert!(ours == inter, "text under .ds-emoji-text is not Inter's");
}
