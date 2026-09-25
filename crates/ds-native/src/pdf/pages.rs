//! A laid-out document painted onto PDF pages: paginated once, then each page painted by
//! Blitz with the viewport scrolled to the page's top, through the krilla painter, under the
//! page's transform (points per pixel, margins) and clipped to its content box.

use crate::pdf::error::PdfError;
use crate::pdf::flow;
use crate::pdf::images;
use crate::pdf::paginate::{Span, paginate};
use crate::pdf::run_texts;
use crate::pdf::spec::{ContentBox, PT_PER_PX};
use anyrender_krilla::{GlyphArea, GlyphTally, KrillaScene, Resources, Sources};
use blitz_dom::BaseDocument;
use blitz_paint::paint_scene;
use krilla::Document;
use krilla::geom::{PathBuilder, Rect as KrillaRect, Transform};
use krilla::page::PageSettings;
use krilla::paint::FillRule;
use peniko::kurbo::Rect;

/// How far above the cut a page's clip ends, in CSS pixels. What the next page starts with
/// begins exactly at the cut, and a viewer that snaps its clip outward to whole device pixels
/// (poppler at 96 dpi) showed a row of it (the top border of a block moved to the next page)
/// at the bottom of this one. Half a pixel of whatever straddles the cut is not drawn instead.
const CLIP_INSET: f32 = 0.5;

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
    let pages = paginate(&flow::collect(doc), content.height);
    let mut pdf = Document::new();
    let mut resources = Resources::new();
    let mut glyphs = GlyphTally::default();
    let scrolled = doc.viewport_scroll();
    for band in &pages {
        let tally = page(&mut pdf, doc, content, *band, &mut resources, sources)?;
        glyphs = add(glyphs, tally);
    }
    doc.set_viewport_scroll(scrolled);
    let bytes = pdf
        .finish()
        .map_err(|error| PdfError::Write(format!("{error:?}")))?;
    Ok(Printed {
        bytes,
        pages,
        glyphs,
    })
}

/// Paint the stretch `band` of `doc` as one page of `pdf`.
fn page(
    pdf: &mut Document,
    doc: &mut BaseDocument,
    content: ContentBox,
    band: Span,
    resources: &mut Resources,
    sources: &Sources,
) -> Result<GlyphTally, PdfError> {
    let (width, height) = content.sheet;
    let settings = PageSettings::from_wh(width.0, height.0).ok_or(PdfError::NoContentArea)?;
    let mut page = pdf.start_page_with(settings);
    let mut surface = page.surface();
    let (left, top) = content.origin;
    surface.push_transform(&Transform::from_row(
        PT_PER_PX, 0.0, 0.0, PT_PER_PX, left.0, top.0,
    ));
    let across = content.width as f32;
    let down = band.height().max(0.0);
    let mut clip = PathBuilder::new();
    let clipped_down = (down - CLIP_INSET).max(f32::EPSILON);
    if let Some(rect) = KrillaRect::from_xywh(0.0, 0.0, across, clipped_down) {
        clip.push_rect(rect);
    }
    let clipped = clip.finish().map(|path| {
        surface.push_clip_path(&path, &FillRule::NonZero);
    });
    doc.set_viewport_scroll(blitz_dom::Point {
        x: 0.0,
        y: f64::from(band.top),
    });
    let area = GlyphArea::Within(Rect::new(0.0, 0.0, f64::from(across), f64::from(down)));
    let tally = {
        let mut scene = KrillaScene::new(&mut surface, resources, sources, area);
        paint_scene(
            &mut scene,
            doc,
            1.0,
            content.width,
            down.ceil() as u32,
            0,
            0,
        );
        scene.tally()
    };
    if clipped.is_some() {
        surface.pop();
    }
    surface.pop();
    surface.finish();
    page.finish();
    Ok(tally)
}

fn add(a: GlyphTally, b: GlyphTally) -> GlyphTally {
    GlyphTally {
        drawn: a.drawn + b.drawn,
        dropped: a.dropped + b.dropped,
        layout_text: a.layout_text + b.layout_text,
        cmap_text: a.cmap_text + b.cmap_text,
    }
}
