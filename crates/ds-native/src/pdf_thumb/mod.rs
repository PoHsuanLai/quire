//! PDF thumbnails from a path (the `pdf-thumb` feature; design/04-COMPONENTS.md section 45).
//!
//! `ds::PdfThumb` draws a page it is handed and never reads a file: ds stays renderer-free and
//! effect-free (`scripts/check-boundary.sh` forbids it pdfrum and every renderer). Reading the
//! file and rasterising its first page are effects, and pdfrum's CPU rasteriser (vello_cpu) is a
//! renderer, so both live here, in quire's one host crate, behind a feature so an app that shows
//! no PDF builds no PDF reader. [`PdfFileThumb`] is what a consumer places: a path and a size in,
//! the raster done on one long-lived worker thread fed by a latest-wins queue (`worker`), the result cached by path, modification time, device
//! size and scale ([`ThumbKey`]), and `ds::PdfThumb` fed each state.

mod cache;
mod raster;
mod request;
mod view;
mod worker;

pub use cache::{THUMB_CACHE_ENTRIES, pdf_thumb_cached};
pub use raster::pdf_thumb_bytes;
pub use request::{DeviceBox, ThumbKey, ThumbRequest, pdf_thumb_blocking, pdf_thumb_rasters};
pub use view::PdfFileThumb;
pub use worker::QUEUE_DEPTH;
