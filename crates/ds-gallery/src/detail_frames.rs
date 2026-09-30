//! `--detail-frames DIR`: design/26's primitives as stills, eight frames through each moment,
//! driven through a `Harness` the way a surface drives them (the state set from outside). Each
//! frame is `DIR/<strip>-<n>.png`, cropped to the specimen at 3x; a script stitches them into
//! strips. It takes a few seconds of wall clock: the moments play in real time.

use crate::details_states::Net;
use crate::details_views::{NetView, ShakeView, SlashView};
use crate::error::GalleryError;
use crate::style;
use dioxus::prelude::*;
use ds::Word;
use ds::detail::{EventStamp, Slashed};
use ds::{Appearance, Ds, Inject, Material, Theme};
use ds_harness::{Driver, Harness, Query, Viewport};
use image::{RgbaImage, imageops};
use std::path::Path;
use std::time::Duration;

/// Which strip a stage draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
enum Strip {
    #[word(slug = "pending-iterate")]
    Pending,
    Shake,
    #[word(slug = "morph-slash")]
    Slash,
}

impl Strip {
    /// How long after the moment starts the first frame is taken, and the step between frames.
    fn timing(self) -> (Duration, Duration) {
        let ms = Duration::from_millis;
        match self {
            // A little into each step (a layer fades over --t-quick).
            Strip::Pending => (ms(40), ms(83)),
            Strip::Shake => (ms(1), ms(55)),
            Strip::Slash => (ms(1), ms(25)),
        }
    }
}

static STRIP: GlobalSignal<Strip> = Signal::global(|| Strip::Pending);
static NET: GlobalSignal<Net> = Signal::global(|| Net::Off);
static SLASHED: GlobalSignal<Slashed> = Signal::global(|| Slashed::Off);

const VIEW: Viewport = Viewport {
    width: 320,
    height: 140,
    scale_percent: 300,
};

#[allow(non_snake_case)] // A component: the harness names it like a type.
fn Stage() -> Element {
    let body = match STRIP() {
        Strip::Pending => rsx! { NetView { net: NET() } },
        Strip::Shake => rsx! { ShakeView { net: NET() } },
        Strip::Slash => rsx! { SlashView { slashed: SLASHED() } },
    };
    rsx! {
        Ds { appearance: Appearance { theme: Theme::Light, ..Appearance::default() }, material: Material::Window, stylesheet: Inject::Inline,
            style { {style::CSS} }
            div { style: "padding:30px", {body} }
        }
    }
}

/// Put the stage in the state before the moment, then start the moment.
fn begin(harness: &mut Harness, strip: Strip) {
    let set = |harness: &mut Harness, net: Net| harness.within(|| *NET.write() = net);
    match strip {
        Strip::Pending => set(harness, Net::Joining),
        Strip::Shake => set(harness, Net::Failed(EventStamp(1))),
        Strip::Slash => harness.within(|| *SLASHED.write() = Slashed::On),
    }
}

/// Eight frames of `strip`'s moment: the stage is switched to the strip, settled, then started.
fn frames(strip: Strip) -> Vec<RgbaImage> {
    let mut harness = Harness::new(Stage, VIEW);
    let (wait, step) = strip.timing();
    harness.within(|| *STRIP.write() = strip);
    harness.advance(Duration::from_millis(200));
    begin(&mut harness, strip);
    harness.advance(wait);
    let scale = f32::from(VIEW.scale_percent) / 100.0;
    let rect = harness.rect(".g-detail");
    let mut shots = Vec::new();
    for _ in 0..8 {
        if let Ok(picture) = harness.render() {
            shots.push(crop(&picture, rect, scale));
        }
        harness.advance(step);
    }
    shots
}

/// `picture` cropped to the specimen with a margin.
fn crop(picture: &RgbaImage, rect: Option<ds::Rect>, scale: f32) -> RgbaImage {
    let Some(rect) = rect else {
        return picture.clone();
    };
    let pad = 12.0;
    let x = ((rect.origin.x.0 - pad).max(0.0) * scale) as u32;
    let y = ((rect.origin.y.0 - pad).max(0.0) * scale) as u32;
    let width = ((rect.size.width.0 + 2.0 * pad) * scale) as u32;
    let height = ((rect.size.height.0 + 2.0 * pad) * scale) as u32;
    imageops::crop_imm(
        picture,
        x,
        y,
        width.min(picture.width().saturating_sub(x)),
        height.min(picture.height().saturating_sub(y)),
    )
    .to_image()
}

/// Write every strip's frames into `dir`.
pub fn run(dir: &Path) -> Result<(), GalleryError> {
    std::fs::create_dir_all(dir).map_err(|source| GalleryError::Write {
        path: dir.to_path_buf(),
        source,
    })?;
    for strip in Strip::ALL.iter().copied() {
        for (index, frame) in frames(strip).iter().enumerate() {
            let path = dir.join(format!("{}-{index}.png", strip.slug()));
            frame.save(&path).map_err(|source| GalleryError::Encode {
                path: path.clone(),
                source,
            })?;
        }
        eprintln!("{}: 8 frames", strip.slug());
    }
    Ok(())
}
