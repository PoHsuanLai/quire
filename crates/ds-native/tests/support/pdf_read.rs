//! Reading a printout back with pdfrum, for assertions on what a reader of the PDF gets:
//! page sizes, extractable text, embedded fonts and images, where each glyph landed.

#![allow(dead_code, reason = "each includer uses its own part of the reader")]

use pdfrum::Document;
use peniko::kurbo::Rect;

/// `bytes` opened as a PDF.
pub fn open(bytes: &[u8]) -> Document {
    Document::from_bytes(bytes.to_vec()).expect("the printout opens as a PDF")
}

/// The text of page `index` as a reader copies it.
pub fn text(doc: &Document, index: u32) -> String {
    doc.page(index).expect("the page exists").text().to_string()
}

/// The text of page `index` with all whitespace removed, so a word the layout broke across
/// lines (CJK wraps anywhere) is still found.
pub fn squeezed(doc: &Document, index: u32) -> String {
    text(doc, index)
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}

/// Every non-space character's box on page `index`, in the page's user space.
pub fn glyph_boxes(doc: &Document, index: u32) -> Vec<(char, Rect)> {
    doc.page(index)
        .expect("the page exists")
        .text()
        .chars
        .iter()
        .filter_map(|glyph| {
            let c = char::from_u32(glyph.unicode)?;
            (!c.is_whitespace()).then_some((c, glyph.char_box))
        })
        .collect()
}

/// How many times `needle` occurs in `haystack`.
pub fn occurrences(haystack: &[u8], needle: &[u8]) -> usize {
    haystack
        .windows(needle.len())
        .filter(|w| *w == needle)
        .count()
}
