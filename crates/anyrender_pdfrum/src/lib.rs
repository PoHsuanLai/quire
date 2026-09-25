//! anyrender scenes written as vector PDF pages through pdfrum: paths stay paths, text stays
//! text in embedded, subsetted faces, and images keep their source encoding where PDF allows (a
//! JPEG as-is, an opaque PNG's compressed data as-is).
//!
//! A page is painted into anyrender's recording [`Scene`](anyrender::Scene) first, by whatever
//! renders through anyrender (Blitz's `paint_scene`). [`write`] then embeds every face and image
//! the scenes use, and replays each scene onto a pdfrum canvas: anyrender's push/pop layers
//! become nested saved graphics states, which is the only shape pdfrum's canvas lets a content
//! stream take.
//!
//! anyrender's glyph runs carry glyph ids and positions, not the text they were shaped from, and
//! a PDF needs the text to make its words selectable. The caller can hand the text over in
//! [`Sources::texts`], keyed by the run as the scene holds it ([`RunKey`]); a run without an
//! entry falls back to the font's own cmap, reversed ([`GlyphTally`] counts both).
//!
//! What is simplified: box shadows are dropped; filters and backdrop filters are ignored;
//! compositing operators other than source-over paint as source-over; a sweep gradient, and a
//! gradient on a stroke or on text, paints as its stops' average colour; gradients pad at their
//! ends whatever extend mode they name, and interpolate in sRGB; an image brush draws once,
//! clipped to its shape (Blitz tiles backgrounds itself).

mod area;
mod cmap;
mod faces;
mod glyphs;
mod paint;
mod prepare;
mod replay;
mod run_text;
mod sources;
mod write;

pub use area::{GlyphArea, GlyphTally};
pub use run_text::{GlyphSource, RunKey, RunText, RunTexts};
pub use sources::{EncodedImage, ImageCodec, ImageSources, Sources};
pub use write::{Page, Written, write};
