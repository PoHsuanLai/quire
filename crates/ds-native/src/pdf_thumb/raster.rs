//! A PDF's first page as PNG pixels, through pdfrum's CPU rasteriser (vello_cpu).

use super::request::DeviceBox;
use ds::{ImageSize, ImageSource, PdfPage, PdfTrouble};
use image::ImageEncoder;
use image::codecs::png::PngEncoder;
use pdfrum::{Document, Error, RenderOptions, VelloCpuBackend};
use std::panic::{AssertUnwindSafe, catch_unwind};

/// `bytes`' first page fitted into `fit`, on white. An empty file, or a document with no
/// pages, is [`PdfPage::Empty`]; a file that is not a PDF or will not draw is
/// [`PdfTrouble::Unreadable`]; one that needs a password is [`PdfTrouble::Locked`]. A panic in
/// the reader on hostile bytes is caught and reads as unreadable.
pub fn pdf_thumb_bytes(bytes: Vec<u8>, fit: DeviceBox) -> PdfPage {
    rasterise(bytes, fit)
}

pub(crate) fn rasterise(bytes: Vec<u8>, fit: DeviceBox) -> PdfPage {
    if bytes.is_empty() {
        return PdfPage::Empty;
    }
    catch_unwind(AssertUnwindSafe(|| first_page(bytes, fit)))
        .unwrap_or(PdfPage::Failed(PdfTrouble::Unreadable))
}

fn first_page(bytes: Vec<u8>, fit: DeviceBox) -> PdfPage {
    let doc = match Document::from_bytes(bytes) {
        Ok(doc) => doc,
        Err(Error::WrongPassword) => return PdfPage::Failed(PdfTrouble::Locked),
        Err(_) => return PdfPage::Failed(PdfTrouble::Unreadable),
    };
    if doc.page_count() == 0 {
        return PdfPage::Empty;
    }
    let Ok(page) = doc.page(0) else {
        return PdfPage::Failed(PdfTrouble::Unreadable);
    };
    let (width, height) = (page.width(), page.height());
    if !(width > 0.0 && height > 0.0) {
        return PdfPage::Failed(PdfTrouble::Unreadable);
    }
    let scale = (f64::from(fit.width) / width).min(f64::from(fit.height) / height);
    let options = RenderOptions::builder()
        .scale(scale)
        .background(peniko::Color::WHITE)
        .build();
    let Ok(pixmap) = page.render_with(VelloCpuBackend, &options) else {
        return PdfPage::Failed(PdfTrouble::Unreadable);
    };
    match png(pixmap.width(), pixmap.height(), &pixmap.to_straight_rgba()) {
        Some(png) => PdfPage::Ready {
            image: ImageSource::png(&png),
            sheet: ImageSize {
                width: width.round() as u32,
                height: height.round() as u32,
            },
        },
        None => PdfPage::Failed(PdfTrouble::Unreadable),
    }
}

/// Straight RGBA as a PNG file's bytes.
fn png(width: u32, height: u32, rgba: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    PngEncoder::new(&mut out)
        .write_image(rgba, width, height, image::ExtendedColorType::Rgba8)
        .ok()?;
    Some(out)
}
