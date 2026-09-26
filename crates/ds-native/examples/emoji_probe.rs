//! The colour-emoji probe (FINDINGS "Colour emoji"): does Blitz at the pinned rev, through
//! parley, skrifa and vello_cpu, paint emoji in colour? Five emoji (a face, a party popper, a
//! thumbs-up with a skin tone, a flag, a ZWJ family) are laid out in rows, each row in one face,
//! on the headless path (`Harness::render`, vello_cpu). Each glyph's cell is measured: how many
//! pixels are inked, and how many of those are coloured (channel spread over 60) rather than
//! grey. Nothing is asserted; the table is the finding.
//!
//! Rows: the system fallback (no emoji family named); `'Noto Color Emoji'` named after Inter in
//! the stack; the installed COLRv1 file (`/usr/share/fonts/google-noto-color-emoji-fonts/
//! Noto-COLRv1.ttf`) registered under "Probe COLRv1"; the monochrome Noto Emoji registered under
//! "Probe Mono"; and, when `EMOJI_CBDT` names a file, Noto Color Emoji CBDT (googlefonts/
//! noto-emoji `2D/fonts/NotoColorEmoji.ttf`, OFL) registered under "Probe CBDT".
//!
//! `cargo run -p ds-native --example emoji_probe -- out.png`

use dioxus::prelude::*;
use ds::{Appearance, Ds, Material};
use ds_native::{Harness, Viewport, font_context};
use parley::fontique::{Blob, FontInfoOverride};
use std::sync::Arc;

const COLRV1: &str = "/usr/share/fonts/google-noto-color-emoji-fonts/Noto-COLRv1.ttf";
const MONO: &str = "/usr/share/fonts/google-noto-emoji-fonts/NotoEmoji-Regular.ttf";
const EMOJI: [&str; 5] = ["😀", "🎉", "👍🏽", "🇹🇼", "👨‍👩‍👧"];

/// A row: its label and its `font-family`.
const ROWS: [(&str, &str); 5] = [
    ("fallback", "'Inter', sans-serif"),
    ("named", "'Inter', 'Noto Color Emoji', sans-serif"),
    ("colrv1", "'Probe COLRv1'"),
    ("mono", "'Probe Mono'"),
    ("cbdt", "'Probe CBDT'"),
];

const VIEW: Viewport = Viewport {
    width: 480,
    height: 400,
    scale_percent: 100,
};

fn register(path: &str, family: &str) -> bool {
    let Ok(bytes) = std::fs::read(path) else {
        return false;
    };
    let mut fonts = font_context();
    let added = fonts.collection.register_fonts(
        Blob::new(Arc::new(bytes) as _),
        Some(FontInfoOverride {
            family_name: Some(family),
            ..FontInfoOverride::default()
        }),
    );
    !added.is_empty()
}

fn app() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "position:absolute; left:0; top:0; background:#fff; width:480px; height:400px;",
                for (row, family) in ROWS {
                    div { key: "{row}", style: "display:flex; height:72px; font-family:{family}; font-size:48px; line-height:72px; color:#000;",
                        for (i, emoji) in EMOJI.iter().enumerate() {
                            span { key: "{i}", id: "{row}-{i}", style: "display:block; width:88px; text-align:center;", "{emoji}" }
                        }
                    }
                }
            }
        }
    }
}

/// Inked and coloured pixels inside `rect`.
fn measure(shot: &image::RgbaImage, rect: ds::Rect) -> (u32, u32) {
    let (x0, y0) = (rect.left().0 as u32, rect.top().0 as u32);
    let (x1, y1) = (
        (rect.left().0 + rect.size.width.0) as u32,
        (rect.top().0 + rect.size.height.0) as u32,
    );
    let mut inked = 0;
    let mut coloured = 0;
    for y in y0..y1.min(shot.height()) {
        for x in x0..x1.min(shot.width()) {
            let [r, g, b, _] = shot.get_pixel(x, y).0;
            if r.min(g).min(b) < 235 {
                inked += 1;
                if r.max(g).max(b) - r.min(g).min(b) > 60 {
                    coloured += 1;
                }
            }
        }
    }
    (inked, coloured)
}

fn main() {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "emoji_probe.png".into());
    println!("registered COLRv1: {}", register(COLRV1, "Probe COLRv1"));
    println!("registered mono: {}", register(MONO, "Probe Mono"));
    if let Ok(cbdt) = std::env::var("EMOJI_CBDT") {
        println!("registered CBDT: {}", register(&cbdt, "Probe CBDT"));
    }
    let mut harness = Harness::new(app, VIEW);
    harness.advance(std::time::Duration::from_millis(200));
    let shot = harness.render().expect("renders");
    shot.save(&out).expect("saved");
    println!("row       {}", EMOJI.map(|e| format!("{e:>14}")).join(""));
    for (row, _) in ROWS {
        let cells: Vec<String> = (0..EMOJI.len())
            .map(|i| match harness.rect(&format!("#{row}-{i}")) {
                Some(rect) => {
                    let (inked, coloured) = measure(&shot, rect);
                    format!("{inked:>7}/{coloured:<6}")
                }
                None => format!("{:>14}", "-"),
            })
            .collect();
        println!("{row:<9} {}", cells.join(""));
    }
    println!("(inked/coloured pixels per 88x72 cell) wrote {out}");
}
