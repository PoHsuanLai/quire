//! `level-motion.png`: the capsule's motion as headless frames (the user's brief of 2026-09-25),
//! driven through a `Harness` the way a person would: a level set from outside sliding over
//! `--t-quick --e-out`, a press swelling the track, a drag past the end stretching the capsule,
//! and the release springing it back. Each frame's title says what it shows and when.

use crate::error::GalleryError;
use crate::level_sheet::stacked;
use crate::pages::level_tile::work;
use crate::style;
use dioxus::prelude::*;
use ds::{
    Appearance, BlurState, Ds, Fraction, Inject, LevelControl, LevelGlyph, Material, Muting, Osd,
    OsdPosition, Point, Px, RootChrome, Shown, Theme,
};
use ds_native::{Harness, Viewport};
use image::{RgbaImage, imageops};
use std::time::Duration;

static VALUE: GlobalSignal<Fraction> = Signal::global(|| Fraction(200));
static CAPTION: GlobalSignal<String> = Signal::global(|| "At rest, 20 %".to_owned());

const VIEW: Viewport = Viewport {
    width: 400,
    height: 150,
    scale_percent: 200,
};
/// Frames per row of the strip.
const PER_ROW: usize = 5;

#[allow(non_snake_case)] // A component: the harness names it like a type.
fn Motion() -> Element {
    rsx! {
        Ds { appearance: Appearance { theme: Theme::Light, ..Appearance::default() }, look: work(Theme::Light), material: Material::Window,
            style { {style::CSS} }
            div { class: "g-level-ground",
                Ds {
                    appearance: Appearance { theme: Theme::Light, ..Appearance::default() },
                    look: work(Theme::Light),
                    material: Material::Osd,
                    chrome: Some(RootChrome::Transparent),
                    blur: BlurState::Unavailable,
                    stylesheet: Inject::Host,
                    Osd { shown: Shown::Visible, label: CAPTION(), position: OsdPosition::BottomCentre,
                        LevelControl {
                            label: "Volume",
                            value: VALUE(),
                            glyph: LevelGlyph::Volume(Muting::Audible),
                            onchange: move |next| *VALUE.write() = next,
                        }
                    }
                }
            }
        }
    }
}

/// The frames: (caption, what to do first, how long to wait before the picture).
fn frames(harness: &mut Harness) -> Vec<RgbaImage> {
    let rail = harness.rect(".ds-level-rail");
    let (left, width, middle) = rail.map_or((40.0, 300.0, 60.0), |rect| {
        (
            rect.origin.x.0,
            rect.size.width.0,
            rect.origin.y.0 + rect.size.height.0 / 2.0,
        )
    });
    let at = |share: f32| Point {
        x: Px(left + width * share),
        y: Px(middle),
    };
    let mut shots = Vec::new();
    let mut shoot = |harness: &mut Harness, caption: String, wait: u64| {
        harness.within(|| *CAPTION.write() = caption);
        harness.advance(Duration::from_millis(wait));
        shots.push(harness.render().ok());
    };
    shoot(harness, "At rest, 20 %".into(), 16);
    harness.within(|| *VALUE.write() = Fraction(800));
    // Each wait is the time since the frame before, so the labels are the time since the change.
    for (label, wait) in [
        ("set to 80 %: 20 ms", 20),
        ("40 ms", 20),
        ("85 ms", 45),
        ("170 ms, settled", 85),
    ] {
        shoot(harness, label.into(), wait);
    }
    harness.pointer_down(at(0.5));
    for (label, wait) in [
        ("press at 50 %: 16 ms", 16),
        ("40 ms", 24),
        ("90 ms", 50),
        ("200 ms, swollen", 110),
    ] {
        shoot(harness, label.into(), wait);
    }
    harness.pointer_move(at(1.0));
    shoot(harness, "drag to the end".into(), 30);
    harness.pointer_move(Point {
        x: Px(left + width + 14.0),
        y: Px(middle),
    });
    shoot(harness, "14 px past: stretched".into(), 30);
    harness.pointer_up(Point {
        x: Px(left + width + 14.0),
        y: Px(middle),
    });
    for (label, wait) in [
        ("released: 30 ms", 30),
        ("80 ms", 50),
        ("160 ms", 80),
        ("300 ms, at rest", 140),
    ] {
        shoot(harness, label.into(), wait);
    }
    shots.into_iter().flatten().collect()
}

/// The strip: every frame cropped to the card, in rows of [`PER_ROW`].
pub fn strip() -> Result<RgbaImage, GalleryError> {
    let mut harness = Harness::new(Motion, VIEW);
    harness.advance(Duration::from_millis(600));
    let scale = f32::from(VIEW.scale_percent) / 100.0;
    let card = harness.rect(".ds-osd");
    let pictures = frames(&mut harness);
    let crop = |picture: &RgbaImage| match card {
        Some(rect) => {
            let pad = 14.0;
            let x = ((rect.origin.x.0 - pad).max(0.0) * scale) as u32;
            let y = ((rect.origin.y.0 - pad).max(0.0) * scale) as u32;
            let w = ((rect.size.width.0 + 2.0 * pad) * scale) as u32;
            let h = ((rect.size.height.0 + 2.0 * pad) * scale) as u32;
            imageops::crop_imm(
                picture,
                x,
                y,
                w.min(picture.width() - x),
                h.min(picture.height() - y),
            )
            .to_image()
        }
        None => picture.clone(),
    };
    let cropped: Vec<RgbaImage> = pictures.iter().map(crop).collect();
    let rows: Vec<RgbaImage> = cropped.chunks(PER_ROW).map(|row| beside(row, 12)).collect();
    if rows.is_empty() {
        return Err(GalleryError::Render {
            name: "level-motion".into(),
            source: ds_native::NativeError::Renderer("no frame".into()),
        });
    }
    Ok(stacked(&rows, 12))
}

/// `pictures` side by side, `gap` apart, on white.
fn beside(pictures: &[RgbaImage], gap: u32) -> RgbaImage {
    let height = pictures.iter().map(RgbaImage::height).max().unwrap_or(1);
    let width = pictures.iter().map(RgbaImage::width).sum::<u32>()
        + gap * pictures.len().saturating_sub(1) as u32;
    let mut row = RgbaImage::from_pixel(width.max(1), height, image::Rgba([255, 255, 255, 255]));
    let mut x = 0i64;
    for picture in pictures {
        imageops::overlay(&mut row, picture, x, 0);
        x += i64::from(picture.width() + gap);
    }
    row
}
