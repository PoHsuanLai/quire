//! The capsule's fill against its well on the light scheme (sill FINDINGS Q105): measured on the
//! painted pixels, the fill and the well at the middle of the track, away from the glyph, as the
//! WCAG contrast ratio of the two colours. Light holds at 1.6:1 or more on every ground the level
//! is drawn on: the paper (a Window root), the control center's module plate (a `ModulePanel`
//! in a painted Popover root) and the OSD card over the Work Space's tint (the level sheet's
//! rows). The dark scheme is printed, not changed.

#[path = "support/probe.rs"]
mod probe;

use dioxus::prelude::*;
use ds::{
    Appearance, BlurState, Ds, Fraction, Inject, Level, LevelControl, LevelGlyph, LevelLook,
    Material, ModulePanel, Muting, Osd, PRESETS, RootChrome, Shown, SpaceLook, Theme,
};
use ds_native::{Harness, Viewport};
use probe::rect;
use std::cell::Cell;
use std::time::Duration;

/// Where the level is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ground {
    Paper,
    Module,
    WorkOsd,
}

thread_local! {
    static CASE: Cell<(Ground, Theme)> = const { Cell::new((Ground::Paper, Theme::Light)) };
}

const VIEW: Viewport = Viewport {
    width: 360,
    height: 200,
    scale_percent: 100,
};

fn work(theme: Theme) -> SpaceLook {
    SpaceLook {
        dots: PRESETS[0].dots.to_vec(),
        theme,
        ..SpaceLook::default()
    }
}

fn level() -> Element {
    rsx! {
        LevelControl { label: "Volume", value: Fraction(500), glyph: LevelGlyph::Volume(Muting::Audible), onchange: |_| {} }
    }
}

#[allow(non_snake_case)]
fn Specimen() -> Element {
    let (ground, theme) = CASE.get();
    let appearance = Appearance {
        theme,
        ..Appearance::default()
    };
    match ground {
        Ground::Paper => rsx! {
            Ds { appearance, material: Material::Window,
                div { style: "width:296px;padding:20px", {level()} }
            }
        },
        Ground::Module => rsx! {
            Ds { appearance, material: Material::Popover, chrome: Some(RootChrome::Painted),
                div { style: "width:296px;padding:20px",
                    ModulePanel { title: "Speakers", {level()} }
                }
            }
        },
        Ground::WorkOsd => rsx! {
            Ds { appearance, look: work(theme), material: Material::Window,
                Ds { appearance, look: work(theme), material: Material::Osd, chrome: Some(RootChrome::Transparent),
                     blur: BlurState::Available, stylesheet: Inject::Host,
                    Osd { shown: Shown::Visible, label: "Sound", look: LevelLook::Capsule,
                          level: Level { value: Fraction(500), glyph: LevelGlyph::Volume(Muting::Audible) } }
                }
            }
        },
    }
}

/// WCAG relative luminance of an sRGB colour.
fn luminance(rgb: [u8; 4]) -> f64 {
    let linear = |c: u8| {
        let c = f64::from(c) / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(rgb[0]) + 0.7152 * linear(rgb[1]) + 0.0722 * linear(rgb[2])
}

fn contrast(a: [u8; 4], b: [u8; 4]) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// The fill's and the well's painted colours and their ratio, for `ground` in `theme`.
fn measure(ground: Ground, theme: Theme) -> ([u8; 4], [u8; 4], f64) {
    CASE.set((ground, theme));
    let mut harness = Harness::new(Specimen, VIEW);
    harness.advance(Duration::from_millis(600));
    let track = rect(&harness, ".ds-level-track");
    let frame = harness.render().expect("renders");
    probe::keep(&frame, &format!("level-contrast-{ground:?}-{theme:?}"));
    let row = (track.origin.y.0 + track.size.height.0 / 2.0) as u32;
    // The glyph sits in the first 30 px; the fill runs to half the track.
    let at = |share: f32| (track.origin.x.0 + track.size.width.0 * share) as u32;
    let fill = frame.get_pixel(at(0.4), row).0;
    let well = frame.get_pixel(at(0.8), row).0;
    (fill, well, contrast(fill, well))
}

#[test]
fn the_light_fill_stands_off_its_well_by_1_6_on_every_ground() {
    let mut failures = Vec::new();
    for ground in [Ground::Paper, Ground::Module, Ground::WorkOsd] {
        let (fill, well, ratio) = measure(ground, Theme::Light);
        println!("light {ground:?}: fill {fill:?} well {well:?} {ratio:.2}:1");
        if ratio < 1.6 {
            failures.push(format!("{ground:?}: {ratio:.2}:1"));
        }
        let (fill, well, ratio) = measure(ground, Theme::Dark);
        println!("dark {ground:?}: fill {fill:?} well {well:?} {ratio:.2}:1");
    }
    assert!(failures.is_empty(), "{}", failures.join(", "));
}
