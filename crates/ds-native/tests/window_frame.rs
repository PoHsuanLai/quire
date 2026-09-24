//! The window's frame on a real Blitz document (FINDINGS "mailo gaps", gap 1): the root paints
//! its two gradient layers and the grain over its own background, not under it. Before the fix
//! a `Material::Window` root was no stacking context, so `.ds-layer` (z -2) and `.ds-grain`
//! (z -1) painted beneath the root's own `background: var(--m-tint-solid)`: a Space switch was
//! an instant swap and the grain never showed.

#[path = "support/probe.rs"]
mod probe;

use dioxus::prelude::*;
use ds::{
    Appearance, Ds, DurationToken, EasingToken, Fraction, Grain, Icon, IconButton,
    IconButtonVariant, Material, MotionLevel, PRESETS, SpaceLook, Theme,
};
use ds_native::{Harness, Viewport};
use image::RgbaImage;
use probe::{distance, keep};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 320,
    height: 200,
    scale_percent: 100,
};

/// The switch sits bottom-right, out of the way of the probed pixels.
const PROBE_CSS: &str =
    ".room{display:flex;justify-content:flex-end;align-items:flex-end;height:200px}";

/// A test override: every frame layer red. If the layers paint above the root's own
/// background, the frame turns red; if they paint beneath it, nothing changes.
const RED_LAYERS: &str = ".ds > .ds-layer{background:#ff0000}";

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn light() -> Appearance {
    Appearance {
        theme: Theme::Light,
        ..Appearance::default()
    }
}

fn look(index: usize, grain: u8) -> SpaceLook {
    SpaceLook {
        dots: PRESETS[index].dots.to_vec(),
        grain: Grain(grain),
        ..SpaceLook::default()
    }
}

fn pixel(frame: &RgbaImage, x: u32, y: u32) -> [u8; 4] {
    frame.get_pixel(x, y).0
}

fn luma(pixel: [u8; 4]) -> i32 {
    (i32::from(pixel[0]) * 3 + i32::from(pixel[1]) * 6 + i32::from(pixel[2])) / 10
}

fn render(app: fn() -> Element) -> RgbaImage {
    let mut harness = Harness::new(app, VIEW);
    harness.advance(ms(60));
    harness.render().expect("renders")
}

fn window(look: SpaceLook, extra: &'static str) -> Element {
    let css = format!("{PROBE_CSS}{extra}");
    rsx! {
        Ds { appearance: light(), look, material: Material::Window,
            style { {css} }
            div { class: "room" }
        }
    }
}

// ---- The layers paint over the root ---------------------------------------------------------

#[allow(non_snake_case)]
fn Plain() -> Element {
    window(look(0, 0), "")
}

#[allow(non_snake_case)]
fn Red() -> Element {
    window(look(0, 0), RED_LAYERS)
}

/// Red layers turn the frame red: the layers are what the window shows.
#[test]
fn the_frame_layers_paint_over_the_root() {
    let plain = render(Plain);
    let red = render(Red);
    keep(&plain, "window-frame-plain");
    keep(&red, "window-frame-red");
    let (at_plain, at_red) = (pixel(&plain, 10, 10), pixel(&red, 10, 10));
    println!("plain {at_plain:?} red layers {at_red:?}");
    assert!(
        distance(at_red, [255, 0, 0, 255]) <= 2,
        "the red layers do not show: {at_red:?} (plain {at_plain:?})"
    );
    assert!(distance(at_plain, at_red) > 60, "{at_plain:?} {at_red:?}");
}

// ---- The grain shows -----------------------------------------------------------------------

#[allow(non_snake_case)]
fn Grainless() -> Element {
    window(look(0, 0), "")
}

#[allow(non_snake_case)]
fn Grainy() -> Element {
    window(look(0, 100), "")
}

/// Grain 100 changes the luminance of the window's pixels where grain 0 leaves the gradient.
#[test]
fn the_grain_changes_the_frame() {
    let smooth = render(Grainless);
    let grainy = render(Grainy);
    keep(&grainy, "window-frame-grain");
    let changed = (0..40u32)
        .flat_map(|y| (0..40u32).map(move |x| (x, y)))
        .map(|(x, y)| (luma(pixel(&smooth, x, y)) - luma(pixel(&grainy, x, y))).abs())
        .collect::<Vec<_>>();
    let moved = changed.iter().filter(|d| **d >= 2).count();
    let most = changed.iter().max().copied().unwrap_or(0);
    println!("grain moved {moved} of 1600 pixels, the most by {most}");
    assert!(
        moved > 100,
        "grain moved {moved} of 1600 pixels (most {most})"
    );
}

// ---- The cross-fade -------------------------------------------------------------------------

#[allow(non_snake_case)]
fn Switching() -> Element {
    let mut index = use_signal(|| 0usize);
    rsx! {
        Ds { appearance: light(), look: look(index(), 0), material: Material::Window,
            style { {PROBE_CSS} }
            div { class: "room",
                IconButton {
                    variant: IconButtonVariant::Status,
                    icon: Icon::Grid,
                    label: "Next Space",
                    onclick: move |_| index.set(3),
                }
            }
        }
    }
}

/// The moment the `--t-scene` / `--e-out` curve reaches a quarter (bar_frame.rs has the same).
fn quarter_of_the_curve() -> Duration {
    let length = DurationToken::Scene.duration(MotionLevel::Standard);
    let curve = EasingToken::Out.easing(MotionLevel::Standard);
    let whole = length.as_millis() as u64;
    let at = (1..=whole)
        .find(|&at| curve.at(Fraction((at * 1000 / whole) as u16)).0 >= 250)
        .unwrap_or(whole / 2);
    ms(at)
}

/// A Space switch cross-fades the window: partway, the corner is neither Space's colour and
/// lies between them (design/21-SPACES.md section 5).
#[test]
fn a_space_switch_cross_fades_the_window() {
    let mut harness = Harness::new(Switching, VIEW);
    harness.advance(ms(60));
    let before = pixel(&harness.render().expect("renders"), 2, 2);
    let button = harness
        .centre(".ds-icon-button")
        .expect("the switch is drawn");
    harness.click(button);
    harness.advance(quarter_of_the_curve());
    let halfway = harness.render().expect("renders");
    keep(&halfway, "window-cross-fade-halfway");
    let middle = pixel(&halfway, 2, 2);
    harness.advance(ms(700));
    let after = pixel(&harness.render().expect("renders"), 2, 2);
    println!("before {before:?} mid-fade {middle:?} after {after:?}");
    assert!(
        distance(before, after) >= 15,
        "the two Spaces differ: {before:?} {after:?}"
    );
    for channel in 0..3 {
        let (low, high) = (
            before[channel].min(after[channel]),
            before[channel].max(after[channel]),
        );
        assert!(
            (low.saturating_sub(3)..=high.saturating_add(3)).contains(&middle[channel]),
            "channel {channel}: mid-fade {middle:?} is not between {before:?} and {after:?}"
        );
    }
    assert!(
        distance(middle, before) > 5 && distance(middle, after) > 5,
        "mid-fade {middle:?} should be neither {before:?} nor {after:?}"
    );
}
