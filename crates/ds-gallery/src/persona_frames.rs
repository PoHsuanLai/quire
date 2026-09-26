//! `--persona-frames DIR`: the persona's motion as stills, eight frames through each mood's
//! motion, driven through a `Harness` the way a lock screen drives it (the mood set from
//! outside). Each frame is `DIR/<mood>-<n>.png`, cropped to the persona at 2x; a script stitches
//! them into strips. Idle's frames are taken around its first blink, which comes 3 to 6 s after
//! the persona mounts, so this takes a few seconds of wall clock.

use crate::error::GalleryError;
use dioxus::prelude::*;
use ds::{Appearance, Ds, Inject, Material, Mood, Persona, PersonaSize, PersonaSpec, Theme};
use ds_native::{Harness, Viewport};
use image::{RgbaImage, imageops};
use std::path::Path;
use std::time::Duration;

/// The seed the strips show.
const SEED: u64 = 17;

static MOOD: GlobalSignal<Mood> = Signal::global(|| Mood::Idle);

const VIEW: Viewport = Viewport {
    width: 200,
    height: 200,
    scale_percent: 200,
};

#[allow(non_snake_case)] // A component: the harness names it like a type.
fn Stage() -> Element {
    rsx! {
        Ds { appearance: Appearance { theme: Theme::Light, ..Appearance::default() }, material: Material::Window, stylesheet: Inject::Inline,
            div { style: "padding:36px",
                Persona { spec: PersonaSpec::from_seed(SEED), size: PersonaSize::Large, mood: MOOD() }
            }
        }
    }
}

/// How a mood's strip is taken: the mood it starts from, how long to wait before the first
/// frame (after the change), and the step between frames.
fn plan(mood: Mood) -> (Mood, Duration, Duration) {
    let ms = Duration::from_millis;
    match mood {
        Mood::Idle => {
            let first = PersonaSpec::from_seed(SEED)
                .blink_gaps()
                .next()
                .unwrap_or(ms(3000));
            (Mood::Idle, first.saturating_sub(ms(20)), ms(25))
        }
        Mood::Attentive => (Mood::Idle, ms(0), ms(25)),
        Mood::Wince | Mood::Happy => (Mood::Idle, ms(0), ms(60)),
        Mood::Asleep => (Mood::Idle, ms(0), ms(340)),
    }
}

/// Eight frames of `mood`'s motion.
fn frames(mood: Mood) -> Vec<RgbaImage> {
    let (from, wait, step) = plan(mood);
    let mut harness = Harness::new(Stage, VIEW);
    harness.within(|| *MOOD.write() = from);
    if mood != Mood::Idle {
        harness.advance(Duration::from_millis(300));
        harness.within(|| *MOOD.write() = mood);
    }
    harness.advance(wait.max(Duration::from_millis(1)));
    let scale = f32::from(VIEW.scale_percent) / 100.0;
    let rect = harness.rect(".ds-persona");
    let mut shots = Vec::new();
    for _ in 0..8 {
        if let Ok(picture) = harness.render() {
            shots.push(crop(&picture, rect, scale));
        }
        harness.advance(step);
    }
    shots
}

/// `picture` cropped to the persona with a margin.
fn crop(picture: &RgbaImage, rect: Option<ds::Rect>, scale: f32) -> RgbaImage {
    let Some(rect) = rect else {
        return picture.clone();
    };
    let pad = 24.0;
    let x = ((rect.origin.x.0 - pad).max(0.0) * scale) as u32;
    let y = ((rect.origin.y.0 - pad).max(0.0) * scale) as u32;
    let side = ((rect.size.width.0 + 2.0 * pad) * scale) as u32;
    imageops::crop_imm(
        picture,
        x,
        y,
        side.min(picture.width() - x),
        side.min(picture.height() - y),
    )
    .to_image()
}

/// Write every mood's frames into `dir`.
pub fn run(dir: &Path) -> Result<(), GalleryError> {
    std::fs::create_dir_all(dir).map_err(|source| GalleryError::Write {
        path: dir.to_path_buf(),
        source,
    })?;
    for mood in Mood::ALL {
        for (index, frame) in frames(mood).iter().enumerate() {
            let path = dir.join(format!("{}-{index}.png", mood.slug()));
            frame.save(&path).map_err(|source| GalleryError::Encode {
                path: path.clone(),
                source,
            })?;
        }
        eprintln!("{}: 8 frames", mood.slug());
    }
    Ok(())
}
