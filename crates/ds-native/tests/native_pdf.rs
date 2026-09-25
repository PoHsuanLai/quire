//! `ds_native::pdf` on the printout fixture (Latin and CJK text, a JPEG and a PNG, a
//! keep-together block placed to straddle page 1's end, a forced break), read back with pdfrum
//! as a reader of the PDF would.

#[path = "support/pdf_read.rs"]
mod pdf_read;
#[path = "support/print_fixture.rs"]
mod print_fixture;

use ds_native::{PageSpec, pdf};
use peniko::kurbo::Rect;
use print_fixture::{BREAK_HEADING, CJK_WORD, KEEP_HEADING, LATIN_WORD, SCREEN_ONLY};
use std::sync::LazyLock;
use std::time::{Duration, Instant};

/// The fixture's JPEG, the printout, and how long a warm print took.
struct Printed {
    jpeg: Vec<u8>,
    bytes: Vec<u8>,
    took: Duration,
}

static PRINTED: LazyLock<Printed> = LazyLock::new(|| {
    let jpeg = print_fixture::jpeg();
    let html = print_fixture::html(&jpeg, &print_fixture::png());
    // The first print in a process also scans the system fonts; time the second.
    pdf(&html, PageSpec::default()).expect("the fixture prints");
    let started = Instant::now();
    let bytes = pdf(&html, PageSpec::default()).expect("the fixture prints");
    Printed {
        jpeg,
        bytes,
        took: started.elapsed(),
    }
});

/// A4 less mailo's margins (18 mm above and below, 16 mm at the sides), in PDF user space
/// (y up), widened by half a point for rounding.
fn content_box() -> Rect {
    let mm = 72.0 / 25.4;
    let (width, height) = (210.0 * mm, 297.0 * mm);
    Rect::new(16.0 * mm, 18.0 * mm, width - 16.0 * mm, height - 18.0 * mm).inflate(0.5, 0.5)
}

#[test]
fn the_fixture_prints_three_a4_pages() {
    let doc = pdf_read::open(&PRINTED.bytes);
    assert_eq!(doc.page_count(), 3);
    for page in doc.pages() {
        assert!(
            (page.width() - 595.28).abs() < 0.1,
            "width {}",
            page.width()
        );
        assert!(
            (page.height() - 841.89).abs() < 0.1,
            "height {}",
            page.height()
        );
    }
}

#[test]
fn latin_and_cjk_faces_are_embedded_and_subsetted() {
    let fonts = pdf_read::open(&PRINTED.bytes).embedded_fonts();
    let named = |part: &str| fonts.iter().find(|font| font.name.contains(part));
    let latin = named("NotoSerif").expect("the Latin serif is embedded");
    let cjk = named("NotoSansCJK").expect("the CJK face is embedded");
    // The CJK face is one variable collection whose default instance is Thin; the subset is
    // cut at the layout's 400 and must say so, not list as Thin in a viewer.
    assert!(
        cjk.name.ends_with("NotoSansCJKtc-Regular"),
        "the CJK subset is named {}",
        cjk.name
    );
    for font in &fonts {
        let tag = font.name.split_once('+').map_or("", |(tag, _)| tag);
        assert!(
            tag.len() == 6 && tag.chars().all(|c| c.is_ascii_uppercase()),
            "{} carries no subset tag",
            font.name
        );
        // Whole faces are hundreds of KB (Noto Serif) to 32 MB (the CJK collection).
        assert!(
            font.data.len() < 40_000,
            "{} is {} bytes",
            font.name,
            font.data.len()
        );
    }
    assert!(
        latin.data.len() < 12_000,
        "Latin subset {} bytes",
        latin.data.len()
    );
    assert!(
        cjk.data.len() < 30_000,
        "CJK subset {} bytes",
        cjk.data.len()
    );
}

#[test]
fn latin_cjk_and_ligatured_words_are_extractable() {
    let doc = pdf_read::open(&PRINTED.bytes);
    let first = pdf_read::squeezed(&doc, 0);
    for word in [LATIN_WORD, CJK_WORD, "field", "一式兩份", "PDF"] {
        assert!(first.contains(word), "{word} not found in page 1: {first}");
    }
    let all: String = (0..doc.page_count())
        .map(|page| pdf_read::text(&doc, page))
        .collect();
    assert!(!all.contains(SCREEN_ONLY), "@media print was not applied");
}

#[test]
fn the_keep_together_block_starts_page_two() {
    let doc = pdf_read::open(&PRINTED.bytes);
    assert!(!pdf_read::text(&doc, 0).contains(KEEP_HEADING));
    let second = pdf_read::text(&doc, 1);
    assert!(second.trim_start().starts_with(KEEP_HEADING), "{second}");
    let (_, first_glyph) = pdf_read::glyph_boxes(&doc, 1)[0];
    let top = content_box().y1;
    assert!(
        top - first_glyph.y1 < 30.0,
        "the block's heading sits {} pt below the content top",
        top - first_glyph.y1
    );
}

#[test]
fn the_forced_break_starts_page_three() {
    let doc = pdf_read::open(&PRINTED.bytes);
    assert!(!pdf_read::text(&doc, 1).contains(BREAK_HEADING));
    let third = pdf_read::text(&doc, 2);
    assert!(third.trim_start().starts_with(BREAK_HEADING), "{third}");
}

#[test]
fn no_glyph_lies_outside_the_content_box() {
    let doc = pdf_read::open(&PRINTED.bytes);
    let area = content_box();
    for page in 0..doc.page_count() {
        let glyphs = pdf_read::glyph_boxes(&doc, page);
        assert!(!glyphs.is_empty(), "page {page} has no text");
        for (c, glyph) in glyphs {
            assert!(
                area.contains(glyph.origin()) && area.contains((glyph.x1, glyph.y1)),
                "page {page}: {c:?} at {glyph:?} is outside {area:?}"
            );
        }
    }
}

#[test]
fn the_jpeg_is_embedded_once_as_it_arrived() {
    let doc = pdf_read::open(&PRINTED.bytes);
    let jpegs: Vec<_> = doc
        .page(0)
        .expect("page 1")
        .images()
        .into_iter()
        .filter_map(|image| image.raw)
        .collect();
    assert_eq!(jpegs.len(), 1, "one image is stored as a JPEG");
    assert_eq!(
        jpegs[0].data, PRINTED.jpeg,
        "the JPEG is the source, byte for byte"
    );
    let signature = &PRINTED.jpeg[..64];
    assert_eq!(pdf_read::occurrences(&PRINTED.bytes, signature), 1);
}

#[test]
fn the_fixture_is_small_and_quick() {
    let size = PRINTED.bytes.len();
    assert!(size < 300_000, "{size} bytes");
    assert!(
        PRINTED.took < Duration::from_millis(500),
        "{:?}",
        PRINTED.took
    );
    eprintln!("fixture: {size} bytes in {:?}", PRINTED.took);
}
