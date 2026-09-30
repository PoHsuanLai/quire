//! Rendering a component headless to pixels (vello CPU), for the gallery's contact sheet and
//! review artefacts. PNGs are review material, never a CI gate: the rasteriser changes with
//! Blitz revisions.
//!
//! A snapshot sees CSS time only: the document is resolved at each moment, but no wall-clock
//! timer runs. Drive timers with [`crate::Harness`].

use crate::driver::Driver;
use crate::error::HarnessError;
use crate::harness::Harness;
use crate::harness_config::HarnessConfig;
use dioxus::prelude::*;
use std::time::Duration;

/// The moment [`snapshot`] renders at: long after every entrance has settled (the longest
/// motion token is well under a second), so the picture is the component at rest.
const AT_REST: Duration = Duration::from_secs(10);

/// The wall-clock time a snapshot lets pass before its first moment, so what a component
/// starts on mount lands in the picture: a resource answered after the first resolve (an image
/// lands one frame late, spike S7) and the one-frame waits quire starts on mount (`FRAME_SLACK`,
/// 34 ms, after an effect's round: the toast and the send pill rise on it). Without it a posed
/// toast was never drawn (gallery fix A).
pub(crate) const MOUNT_SETTLE: Duration = Duration::from_millis(120);

/// How many `MOUNT_SETTLE` rounds a snapshot waits before its first moment: a placement takes
/// a few (measure the target, render, measure the surface, place).
const SETTLE_ROUNDS: u32 = 8;

/// The size and scale a snapshot renders at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Viewport {
    /// Width in logical pixels.
    pub width: u32,
    /// Height in logical pixels.
    pub height: u32,
    /// Device pixels per logical pixel, in hundredths: 100 is 1x, 200 is 2x.
    pub scale_percent: u16,
}

/// Render `app` once, after its fonts and images have landed, to an RGBA image.
pub fn snapshot(
    app: fn() -> Element,
    viewport: Viewport,
) -> Result<image::RgbaImage, HarnessError> {
    let mut frames = snapshot_at(app, viewport, &[AT_REST])?;
    frames
        .pop()
        .ok_or_else(|| HarnessError::Renderer("no frame rendered".into()))
}

/// Render `app` at each of `moments` of animation time (measured from its first frame, in
/// order), after its fonts and images have landed and its mount-time waits have run: one image
/// per moment, for pictures of a motion part-way through. Each moment's CSS time is exact; the
/// settle only lets timers and fetches land first.
pub fn snapshot_at(
    app: fn() -> Element,
    viewport: Viewport,
    moments: &[Duration],
) -> Result<Vec<image::RgbaImage>, HarnessError> {
    snapshot_with(app, HarnessConfig::new(viewport), moments)
}

/// As [`snapshot_at`], with the document built as `config` says: the app's own contexts and
/// providers, so a component that reads `use_context` renders as it does in the window.
pub fn snapshot_with(
    app: fn() -> Element,
    config: HarnessConfig,
    moments: &[Duration],
) -> Result<Vec<image::RgbaImage>, HarnessError> {
    snapshot_rounds(app, config, moments, 1)
}

/// As [`snapshot_at`], after waiting for placements to land: a hint or popover is hidden until
/// its target and its own size are measured, each a frame after the last, so a page with such
/// surfaces (the gallery's) waits several settle rounds before its first moment.
pub fn snapshot_placed(
    app: fn() -> Element,
    viewport: Viewport,
    moments: &[Duration],
) -> Result<Vec<image::RgbaImage>, HarnessError> {
    snapshot_rounds(app, HarnessConfig::new(viewport), moments, SETTLE_ROUNDS)
}

fn snapshot_rounds(
    app: fn() -> Element,
    config: HarnessConfig,
    moments: &[Duration],
    rounds: u32,
) -> Result<Vec<image::RgbaImage>, HarnessError> {
    let mut harness = Harness::new(app, config);
    for _ in 0..rounds {
        harness.advance(MOUNT_SETTLE);
    }
    moments
        .iter()
        .map(|&moment| harness.render_at(moment))
        .collect()
}
