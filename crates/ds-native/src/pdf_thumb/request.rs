//! What a thumbnail is asked for, and the key its raster is cached under.

use super::cache;
use super::raster::rasterise;
use ds::{PdfPage, PdfTrouble, Scale, Size};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

/// The largest side a thumbnail is rasterised at, in device pixels: a preview pane at 3x, and a
/// cap so a huge `size` cannot ask vello_cpu for a poster.
const LARGEST_SIDE: u32 = 2048;

/// A thumbnail wanted: the file, the room it is fitted into (logical pixels), and the device
/// scale it is drawn at.
#[derive(Debug, Clone, PartialEq)]
pub struct ThumbRequest {
    /// The PDF.
    pub path: PathBuf,
    /// The room the page is fitted into, as `ds::PdfThumb`'s `size`.
    pub size: Size,
    /// The device scale (`ds::use_scale()`).
    pub scale: Scale,
}

/// A room in whole device pixels: the most a raster may cover.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeviceBox {
    /// Pixels across.
    pub width: u32,
    /// Pixels down.
    pub height: u32,
}

/// What a raster is cached under: the same file unchanged, at the same device size and scale,
/// is the same picture. A file rewritten in place gets a new modification time and misses.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ThumbKey {
    /// The file.
    pub path: PathBuf,
    /// Its modification time when it was read.
    pub modified: SystemTime,
    /// The room in device pixels.
    pub fit: DeviceBox,
    /// The scale that room was taken at.
    pub scale: Scale,
}

impl ThumbRequest {
    /// The room in device pixels, each side rounded up and held to 1..=2048.
    pub fn device_box(&self) -> DeviceBox {
        let side = |px: f32| {
            let device = (f64::from(px) * self.scale.as_f64()).ceil();
            (device.clamp(1.0, f64::from(LARGEST_SIDE))) as u32
        };
        DeviceBox {
            width: side(self.size.width.0),
            height: side(self.size.height.0),
        }
    }

    /// The cache key, reading the file's modification time; an unreadable file (missing, no
    /// permission) is [`PdfTrouble::Unreadable`].
    pub fn key(&self) -> Result<ThumbKey, PdfTrouble> {
        let modified = std::fs::metadata(&self.path)
            .and_then(|meta| meta.modified())
            .map_err(|_| PdfTrouble::Unreadable)?;
        Ok(ThumbKey {
            path: self.path.clone(),
            modified,
            fit: self.device_box(),
            scale: self.scale,
        })
    }
}

/// The request's page, from the cache or rasterised now on the calling thread, and cached.
/// Blocks for the read and the raster: call it off the UI thread ([`super::PdfFileThumb`] does).
pub fn pdf_thumb_blocking(request: &ThumbRequest) -> PdfPage {
    let key = match request.key() {
        Ok(key) => key,
        Err(trouble) => return PdfPage::Failed(trouble),
    };
    if let Some(hit) = cache::lookup(&key) {
        return hit;
    }
    RASTERS.fetch_add(1, Ordering::Relaxed);
    let page = match std::fs::read(&key.path) {
        Ok(bytes) => rasterise(bytes, key.fit),
        Err(_) => PdfPage::Failed(PdfTrouble::Unreadable),
    };
    cache::insert(key, page.clone());
    page
}

/// Files read and rasterised (cache misses) so far in this process.
static RASTERS: AtomicU64 = AtomicU64::new(0);

/// How many pages this process has read and rasterised, cache hits not counted: a diagnostic,
/// and how a test proves superseded requests never ran.
pub fn pdf_thumb_rasters() -> u64 {
    RASTERS.load(Ordering::Relaxed)
}
