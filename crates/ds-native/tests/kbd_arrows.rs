//! Sheet and modal parts, Q95: a Small key cap's Left and Right arrows are arrows, not dashes.
//! Extended for sill Q111: the arrows the font subset now draws are real arrows but only ~4 px
//! of ink, still close to a dash. `data-glyph="arrow"` (`Key::glyph_kind`) draws Up, Down, Left
//! and Right at `--fs-control` with a tighter line-height, so the ink is at least 7 px wide
//! while the cap's own box (padding and border drive its height, not the glyph) stays the
//! height every other Small cap already has.
//!
//! Space Mono's latin subset (Google's `unicode-range`) carried `↑` and `↓` but not `←` and `→`,
//! so the two fell back to a system face whose arrows at 9.5 px are a thin stroke with a head a
//! pixel tall. The subset now carries U+2190-2193 from the same Space Mono release, so every
//! arrow is drawn by the mono face. The probe is the ink's bounding box inside the cap: an
//! arrow's head makes the ink as tall as a third of its width, a dash is one pixel tall.

#[path = "support/probe.rs"]
mod probe;

use dioxus::prelude::*;
use ds::{Appearance, Ds, Kbd, KbdSize, Key, Material, Shortcut};
use ds_native::{Harness, Viewport};
use image::RgbaImage;
use probe::{distance, modal, rect};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 240,
    height: 80,
    scale_percent: 100,
};

/// One Small cap per arrow, plus a plain Small letter cap (`plain`) as the height baseline every
/// arrow cap must still match, each in its own box so each can be probed alone.
#[allow(non_snake_case)]
fn Page() -> Element {
    let keys = [
        ("left", Key::Left),
        ("right", Key::Right),
        ("up", Key::Up),
        ("down", Key::Down),
        ("plain", Key::Char('t')),
    ];
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "padding:20px; display:flex; gap:12px; align-items:flex-start",
                for (name, key) in keys {
                    span { key: "{name}", class: "{name}",
                        Kbd { shortcut: Shortcut(vec![key]), size: KbdSize::Small }
                    }
                }
            }
        }
    }
}

/// The ink's width and height inside `selector`'s cap, in device pixels: the box around every
/// pixel that stands out from the cap's own face by more than a faint anti-alias.
fn ink_box(harness: &Harness, frame: &RgbaImage, selector: &str) -> (u32, u32) {
    let cap = rect(harness, selector);
    // Inside the cap's border (1 px, 2 px at the bottom) so the rim is not ink.
    let (x0, y0) = (cap.origin.x.0 as u32 + 2, cap.origin.y.0 as u32 + 2);
    let x1 = (cap.origin.x.0 + cap.size.width.0) as u32 - 2;
    let y1 = (cap.origin.y.0 + cap.size.height.0) as u32 - 3;
    let seen: Vec<(u32, u32, [u8; 4])> = (y0..y1)
        .flat_map(|y| (x0..x1).map(move |x| (x, y)))
        .map(|(x, y)| (x, y, frame.get_pixel(x, y).0))
        .collect();
    let face = modal(&seen.iter().map(|(_, _, pixel)| *pixel).collect::<Vec<_>>());
    let ink: Vec<(u32, u32)> = seen
        .iter()
        .filter(|(_, _, pixel)| distance(*pixel, face) > 60)
        .map(|(x, y, _)| (*x, *y))
        .collect();
    let span = |values: Vec<u32>| {
        let low = values.iter().min().copied().unwrap_or(0);
        let high = values.iter().max().copied().unwrap_or(0);
        if values.is_empty() { 0 } else { high - low + 1 }
    };
    (
        span(ink.iter().map(|(x, _)| *x).collect()),
        span(ink.iter().map(|(_, y)| *y).collect()),
    )
}

#[test]
fn a_small_caps_left_and_right_arrows_have_heads() {
    let mut harness = Harness::new(Page, VIEW);
    harness.advance(Duration::from_millis(50));
    let frame = harness.render().expect("the page renders");
    probe::keep(&frame, "kbd-arrows");
    let (up_w, up_h) = ink_box(&harness, &frame, ".up .ds-kbd");
    assert!(up_h >= 5, "the up arrow is drawn: {up_w} x {up_h}");
    for side in [".left .ds-kbd", ".right .ds-kbd"] {
        let (width, height) = ink_box(&harness, &frame, side);
        // Q111: 5 px still read as a dash; the arrow face (`--fs-control`) asks for at least 7.
        assert!(width >= 7, "{side}: the arrow is {width} px long");
        assert!(
            height >= 3 && height * 3 >= width,
            "{side}: the ink is {width} x {height}, a dash rather than an arrow"
        );
        // Drawn by the same face as the up arrow: its length is the up arrow's height, give or
        // take the pixel grid.
        assert!(
            width.abs_diff(up_h) <= 2,
            "{side}: {width} px long against the up arrow's {up_h} px"
        );
    }
}

/// Q111: the bigger arrow face's tighter line-height keeps the cap's own box the height every
/// other Small cap already has (`plain`, an unstyled Small letter cap) — only the glyph inside
/// grows, not the rim around it.
#[test]
fn a_small_arrow_caps_box_is_the_same_height_as_a_plain_small_cap() {
    let mut harness = Harness::new(Page, VIEW);
    harness.advance(Duration::from_millis(50));
    harness.render().expect("the page renders");
    let plain_height = rect(&harness, ".plain .ds-kbd").size.height.0;
    for side in [
        ".left .ds-kbd",
        ".right .ds-kbd",
        ".up .ds-kbd",
        ".down .ds-kbd",
    ] {
        let height = rect(&harness, side).size.height.0;
        assert!(
            (height - plain_height).abs() < 1.0,
            "{side}: cap is {height} px tall against a plain Small cap's {plain_height} px"
        );
    }
}
