//! An [`anyrender::PaintScene`] that writes into a krilla PDF page, so anything that paints
//! through anyrender (Blitz's `paint_scene`) can produce a vector PDF: paths stay paths, text
//! stays text in embedded, subsetted fonts, and images keep their source encoding where krilla
//! allows (a JPEG is written as-is).
//!
//! anyrender's `draw_glyphs` carries glyph ids and positions, not the text they were shaped
//! from, and a PDF needs the text to make its words selectable. The caller can hand the text
//! over in [`Sources::texts`], keyed by the run as the painter will see it ([`RunKey`]); a run
//! without an entry falls back to the font's own cmap, reversed ([`GlyphTally`] counts both).
//!
//! What is simplified: box shadows are dropped; filters and backdrop filters are ignored;
//! compositing operators other than source-over paint as source-over; gradients interpolate in
//! sRGB whatever colour space they name; an image brush draws once, clipped to the shape,
//! whatever its extend mode (Blitz tiles backgrounds itself).

mod cmap;
mod fonts;
mod geometry;
mod glyphs;
mod gradient;
mod image;
mod instance;
mod layer;
mod paint;
mod resources;
mod run_text;
mod scene;
mod sources;

pub use glyphs::{GlyphArea, GlyphTally};
pub use resources::Resources;
pub use run_text::{GlyphSource, RunKey, RunText, RunTexts};
pub use scene::KrillaScene;
pub use sources::{EncodedImage, ImageCodec, ImageSources, Sources};
