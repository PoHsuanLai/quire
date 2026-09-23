//! Rendering a component headless to pixels (vello CPU), for the gallery's contact sheet and
//! review artefacts. PNGs are review material, never a CI gate: the rasteriser changes with
//! Blitz revisions.
#![allow(unused_variables, dead_code)] // Freeze stubs: remove with the last todo!().

use crate::error::NativeError;
use dioxus::prelude::*;

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
pub fn snapshot(app: fn() -> Element, viewport: Viewport) -> Result<image::RgbaImage, NativeError> {
    todo!()
}
