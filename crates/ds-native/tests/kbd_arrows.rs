//! Sheet and modal parts, Q95: a Small key cap's Left and Right arrows are arrows, not dashes.
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
    width: 200,
    height: 80,
    scale_percent: 100,
};

/// One Small cap per arrow, each in its own box so each can be probed alone.
#[allow(non_snake_case)]
fn Page() -> Element {
    let keys = [
        ("left", Key::Left),
        ("right", Key::Right),
        ("up", Key::Up),
        ("down", Key::Down),
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
        assert!(width >= 5, "{side}: the arrow is {width} px long");
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
