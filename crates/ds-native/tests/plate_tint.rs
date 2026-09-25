//! A tinted plate on a real Blitz document (sill FINDINGS Q72): a Neutral plate with
//! `plate_tint: PlateTint::of(Monochrome, Tint::space(..))` paints its gradient in the tint's hue
//! in the dark scheme and the light one, read back from the pixels, and a plate without a tint
//! paints the family's stops exactly as before.

#[path = "support/probe.rs"]
mod probe;

use dioxus::prelude::*;
use ds::icon::{IconStyle, Tint};
use ds::{
    Appearance, Ds, Icon, IconSize, IconSource, IconView, Material, PRESETS, PlateFamily,
    PlateTint, Scheme, Surface,
};
use ds_native::{Harness, Viewport};
use image::{Rgba, RgbaImage};
use probe::{keep, rect};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 520,
    height: 300,
    scale_percent: 100,
};

/// The Space presets whose tints are checked: Work and Home.
const SPACES: [usize; 2] = [0, 1];

fn tint(space: usize) -> Tint {
    Tint::space(PRESETS[space].dots)
}

/// Row `scheme`, each Space's tinted plate then an untinted one. A hollow glyph (`Square`)
/// leaves the plate's centre to the gradient.
#[allow(non_snake_case)]
fn Plates() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            for scheme in Scheme::ALL {
                div { class: "{scheme.slug()}", style: "display:flex; gap:24px; padding:20px",
                    Surface { material: Material::Popover, theme: Some(scheme),
                        div { style: "display:flex; gap:24px; padding:8px",
                            for space in SPACES {
                                span { class: "tinted-{space}",
                                    IconView { source: IconSource::Glyph(Icon::Square), size: IconSize::Tile96, plate: Some(PlateFamily::Neutral), plate_tint: PlateTint::of(IconStyle::Monochrome, tint(space)) }
                                }
                            }
                            span { class: "plain",
                                IconView { source: IconSource::Glyph(Icon::Square), size: IconSize::Tile96, plate: Some(PlateFamily::Neutral) }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The pixel at `(dx, dy)` in the plate under `.<scheme> .<which>`.
fn at(
    frame: &RgbaImage,
    harness: &Harness,
    scheme: Scheme,
    which: &str,
    d: (f32, f32),
) -> Rgba<u8> {
    let plate = rect(harness, &format!(".{} .{which} .ds-plate", scheme.slug()));
    *frame.get_pixel(
        (plate.origin.x.0 + d.0) as u32,
        (plate.origin.y.0 + d.1) as u32,
    )
}

/// A pixel's OKLCh hue and chroma (chroma capped at 0.07, which none of these reach).
fn lch(pixel: Rgba<u8>) -> Tint {
    Tint::from_hex(&format!(
        "#{:02x}{:02x}{:02x}",
        pixel[0], pixel[1], pixel[2]
    ))
    .expect("hex")
}

fn hue_gap(a: f32, b: f32) -> f32 {
    ((a - b + 540.0).rem_euclid(360.0) - 180.0).abs()
}

/// The plate's centre (inside the hollow glyph) and its deep corner (inside the superellipse,
/// outside the glyph).
const CENTRE: (f32, f32) = (48.0, 48.0);
const DEEP: (f32, f32) = (84.0, 84.0);

/// How near the tint's hue a stop must read. The dark paper takes the tint at chroma about
/// .05, which eight bits resolve to within 2 degrees; the light paper sits at lightness .96-1,
/// where the tint eases to chroma .004-.009 and a unit of rounding moves the hue up to 8
/// degrees (measured 2026-09-25: Work 268 reads 275 at the centre, Home 152 reads 157).
fn tolerance(scheme: Scheme) -> (f32, f32) {
    match scheme {
        Scheme::Dark => (8.0, 0.03),
        Scheme::Light => (12.0, 0.003),
    }
}

#[test]
fn a_tinted_neutral_plate_takes_the_spaces_hue_in_both_schemes() {
    let mut harness = Harness::new(Plates, VIEW);
    harness.advance(Duration::from_millis(60));
    let frame = harness.render().expect("a frame");
    keep(&frame, "plate_tint");
    for scheme in Scheme::ALL {
        let (hue_tolerance, least_chroma) = tolerance(scheme);
        for space in SPACES {
            let want = tint(space);
            for (name, d) in [("centre", CENTRE), ("deep corner", DEEP)] {
                let pixel = at(&frame, &harness, scheme, &format!("tinted-{space}"), d);
                let got = lch(pixel);
                assert!(
                    hue_gap(got.hue, want.hue) < hue_tolerance && got.chroma > least_chroma,
                    "{scheme:?} space {space} {name}: {pixel:?} reads {got:?}, tint {want:?}"
                );
            }
        }
    }
}

/// Without a tint the plate is the family's CSS gradient: the dark paper `#2A2E28` to
/// `#1D211B` and the light `#FFFFFF` to `#F1F3EE`, interpolated in sRGB, so the centre is their
/// mean and the deep corner lies between the mean and the deep stop. (The light paper's own
/// hue, about 125, is too near Home's 152 for a hue test to tell them apart; the stops are.)
#[test]
fn an_untinted_plate_is_unchanged() {
    let mut harness = Harness::new(Plates, VIEW);
    harness.advance(Duration::from_millis(60));
    let frame = harness.render().expect("a frame");
    for (scheme, base, deep) in [
        (Scheme::Dark, [0x2Au8, 0x2E, 0x28], [0x1Du8, 0x21, 0x1B]),
        (Scheme::Light, [0xFFu8, 0xFF, 0xFF], [0xF1u8, 0xF3, 0xEE]),
    ] {
        let centre = at(&frame, &harness, scheme, "plain", CENTRE);
        let corner = at(&frame, &harness, scheme, "plain", DEEP);
        for i in 0..3 {
            let mean = (i32::from(base[i]) + i32::from(deep[i])) / 2;
            assert!(
                (i32::from(centre[i]) - mean).abs() <= 2,
                "{scheme:?} centre {centre:?}, mean of the stops {mean} in channel {i}"
            );
            let (low, high) = (mean.min(i32::from(deep[i])), mean.max(i32::from(deep[i])));
            assert!(
                (low - 2..=high + 2).contains(&i32::from(corner[i])),
                "{scheme:?} deep corner {corner:?} outside {low}..{high} in channel {i}"
            );
        }
    }
}
