//! The painter on its own, without Blitz: a glyph run drawn onto a krilla page, read back.

use anyrender::{Glyph, PaintScene};
use anyrender_krilla::{GlyphArea, GlyphSource, KrillaScene, Resources, RunKey, RunText, Sources};
use krilla::Document;
use krilla::page::PageSettings;
use peniko::kurbo::{Affine, Rect, Vec2};
use peniko::{Blob, Color, Fill, FontData};
use skrifa::{FontRef, MetadataProvider as _};
use std::sync::{Arc, LazyLock};

/// quire's Noto Serif (Latin subset), a static-at-default variable face.
static SERIF: &[u8] = include_bytes!("../../ds/assets/fonts/noto-serif-normal-400-700-latin.ttf");

/// The face as a renderer holds it: one blob, whose id names the face in every run key.
static FONT: LazyLock<FontData> = LazyLock::new(|| FontData::new(Blob::new(Arc::new(SERIF)), 0));

/// `text`'s glyphs in the serif at 20 px from x = 10, one per character, advanced by the face.
fn glyphs(text: &str) -> Vec<Glyph> {
    let face = FontRef::new(SERIF).expect("the face parses");
    let metrics = face.glyph_metrics(
        skrifa::instance::Size::new(20.0),
        skrifa::instance::LocationRef::default(),
    );
    let mut x = 10.0;
    text.chars()
        .map(|c| {
            let id = face.charmap().map(c).expect("the face has the character");
            let glyph = Glyph {
                id: id.to_u32(),
                x,
                y: 40.0,
            };
            x += metrics.advance_width(id).unwrap_or(10.0);
            glyph
        })
        .collect()
}

/// Paint `run` (with `sources`) onto a 200 x 100 pt page, keeping glyphs `area` holds.
fn paint(
    run: &[Glyph],
    sources: &Sources,
    area: GlyphArea,
) -> (Vec<u8>, anyrender_krilla::GlyphTally) {
    let mut pdf = Document::new();
    let mut resources = Resources::new();
    let mut page = pdf.start_page_with(PageSettings::from_wh(200.0, 100.0).expect("a size"));
    let mut surface = page.surface();
    let tally = {
        let mut scene = KrillaScene::new(&mut surface, &mut resources, sources, area);
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            Color::from_rgb8(200, 220, 255),
            None,
            &Rect::new(0.0, 0.0, 200.0, 100.0),
        );
        scene.draw_glyphs(
            &FONT,
            20.0,
            false,
            &[],
            Vec2::ZERO,
            Fill::NonZero,
            Color::BLACK,
            1.0,
            Affine::IDENTITY,
            None,
            run.iter().copied(),
        );
        scene.tally()
    };
    surface.finish();
    page.finish();
    (pdf.finish().expect("the page writes"), tally)
}

fn text(bytes: &[u8]) -> String {
    let doc = pdfrum::Document::from_bytes(bytes.to_vec()).expect("opens");
    doc.page(0u32)
        .expect("page 1")
        .text()
        .to_string()
        .trim()
        .to_owned()
}

#[test]
fn a_run_prints_the_text_it_was_given() {
    let run = glyphs("Hello");
    let mut sources = Sources::default();
    // Deliberately not what the glyphs spell, so the text can only have come from the caller.
    let given = RunText::from_glyphs("HELLO".char_indices().map(|(at, _)| GlyphSource {
        cluster: at,
        text: &"HELLO"[at..at + 1],
    }));
    sources
        .texts
        .insert(RunKey::new(&FONT, 20.0, run.iter().copied()), given);
    let (bytes, tally) = paint(&run, &sources, GlyphArea::Anywhere);
    assert_eq!(text(&bytes), "HELLO");
    assert_eq!((tally.drawn, tally.layout_text, tally.cmap_text), (5, 5, 0));
}

#[test]
fn a_run_without_text_falls_back_to_the_cmap() {
    let (bytes, tally) = paint(&glyphs("Hello"), &Sources::default(), GlyphArea::Anywhere);
    assert_eq!(text(&bytes), "Hello");
    assert_eq!((tally.drawn, tally.cmap_text), (5, 5));
}

#[test]
fn glyphs_outside_the_area_are_not_written() {
    // The area ends 34 px in: "He" (box centres near 17 and 30) stays, "llo" goes.
    let area = GlyphArea::Within(Rect::new(0.0, 0.0, 34.0, 100.0));
    let (bytes, tally) = paint(&glyphs("Hello"), &Sources::default(), area);
    assert_eq!(text(&bytes), "He");
    assert_eq!((tally.drawn, tally.dropped), (2, 3));
}
