//! A laid-out document painted onto PDF pages: paginated once, then each page recorded by
//! Blitz with the viewport scrolled to the page's top, and the recordings written through the
//! pdfrum painter under each page's placement (points per pixel, the flip to PDF's y-up, the
//! margins), clipped to its content box.

use crate::pdf::error::PdfError;
use crate::pdf::flow;
use crate::pdf::images;
use crate::pdf::paginate::{Span, paginate};
use crate::pdf::run_texts;
use crate::pdf::spec::{ContentBox, PT_PER_PX};
use anyrender::Scene;
use anyrender_pdfrum::{GlyphArea, GlyphTally, Page, Sources};
use blitz_dom::BaseDocument;
use blitz_paint::paint_scene;
use peniko::kurbo::{Affine, Rect, Size};

/// How far above the cut a page's clip ends, in CSS pixels. What the next page starts with
/// begins exactly at the cut, and a viewer that snaps its clip outward to whole device pixels
/// (poppler at 96 dpi) showed a row of it (the top border of a block moved to the next page)
/// at the bottom of this one. Half a pixel of whatever straddles the cut is not drawn instead.
const CLIP_INSET: f64 = 0.5;

/// A finished PDF and what went into it.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Printed {
    pub(crate) bytes: Vec<u8>,
    pub(crate) pages: Vec<Span>,
    pub(crate) glyphs: GlyphTally,
}

/// `doc`, laid out at `content`'s width, as a PDF on `content`'s sheet.
pub(crate) fn print(doc: &mut BaseDocument, content: ContentBox) -> Result<Printed, PdfError> {
    let sources = Sources {
        texts: run_texts::collect(doc),
        images: images::collect(doc),
    };
    print_from(doc, content, &sources)
}

/// As [`print`], with the run texts and image sources given rather than read from `doc`.
pub(crate) fn print_from(
    doc: &mut BaseDocument,
    content: ContentBox,
    sources: &Sources,
) -> Result<Printed, PdfError> {
    let bands = paginate(&flow::collect(doc), content.height);
    let scrolled = doc.viewport_scroll();
    let pages: Vec<Page> = bands.iter().map(|band| page(doc, content, *band)).collect();
    doc.set_viewport_scroll(scrolled);
    let written = anyrender_pdfrum::write(&pages, sources)
        .map_err(|error| PdfError::Write(error.to_string()))?;
    Ok(Printed {
        bytes: written.bytes,
        pages: bands,
        glyphs: written.glyphs,
    })
}

/// The stretch `band` of `doc`, recorded, and where it goes on its sheet.
fn page(doc: &mut BaseDocument, content: ContentBox, band: Span) -> Page {
    let (width, height) = content.sheet;
    let (left, top) = content.origin;
    let across = f64::from(content.width);
    let down = f64::from(band.height().max(0.0));
    doc.set_viewport_scroll(blitz_dom::Point {
        x: 0.0,
        y: f64::from(band.top),
    });
    let mut scene = Scene::new();
    paint_scene(
        &mut scene,
        doc,
        1.0,
        content.width,
        down.ceil() as u32,
        0,
        0,
    );
    let scale = f64::from(PT_PER_PX);
    Page {
        size: Size::new(f64::from(width.0), f64::from(height.0)),
        scene,
        // CSS px, y down, from the content box's corner -> points, y up, from the sheet's.
        placement: Affine::new([
            scale,
            0.0,
            0.0,
            -scale,
            f64::from(left.0),
            f64::from(height.0 - top.0),
        ]),
        clip: Rect::new(0.0, 0.0, across, (down - CLIP_INSET).max(0.0)),
        area: GlyphArea::Within(Rect::new(0.0, 0.0, across, down)),
    }
}
