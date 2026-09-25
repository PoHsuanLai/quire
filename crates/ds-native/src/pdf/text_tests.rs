//! Where a printout's text comes from, shown on the two cases the font's cmap gets wrong: a
//! ligature, and a glyph two code points share. Each is printed twice, with the layout's run
//! texts (what `pdf` does) and with none (the reversed-cmap fallback every run without an
//! entry gets), and read back with pdfrum.

use crate::pdf::html;
use crate::pdf::pages::{self, Printed};
use crate::pdf::run_texts;
use crate::pdf::spec::{ContentBox, PageSpec};
use anyrender_krilla::Sources;

/// Which text source a print uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Texts {
    Layout,
    CmapOnly,
}

fn print(body: &str, texts: Texts) -> (Printed, String) {
    let content = ContentBox::of(PageSpec::default()).expect("A4 has room");
    let markup = format!(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"></head><body \
         style=\"margin:0\">{body}</body></html>"
    );
    let mut doc = html::document(&markup, content);
    let sources = Sources {
        texts: match texts {
            Texts::Layout => run_texts::collect(&doc),
            Texts::CmapOnly => Default::default(),
        },
        images: Default::default(),
    };
    let printed = pages::print_from(&mut doc, content, &sources).expect("prints");
    let read = pdfrum::Document::from_bytes(printed.bytes.clone()).expect("opens");
    let text = read.page(0).expect("page 1").text().to_string();
    (
        printed,
        text.split_whitespace().collect::<Vec<_>>().join(" "),
    )
}

const LIGATURES: &str = "<p style=\"font: 20px 'Noto Serif'\">field office flat</p>";

#[test]
fn a_ligature_copies_as_its_letters_from_the_layout() {
    let (printed, text) = print(LIGATURES, Texts::Layout);
    assert_eq!(text, "field office flat");
    assert_eq!(printed.glyphs.cmap_text, 0, "every glyph had layout text");
    assert!(printed.glyphs.layout_text > 0);
}

#[test]
fn the_cmap_fallback_cannot_spell_a_ligature() {
    // Noto Serif draws `fi`, `ffi` and `fl` as one glyph each; the glyph has no cmap entry of
    // its own (or only U+FB01-style presentation forms), so read backwards it loses letters.
    let (printed, text) = print(LIGATURES, Texts::CmapOnly);
    assert_eq!(printed.glyphs.layout_text, 0);
    assert_ne!(
        text, "field office flat",
        "the fallback was expected to be wrong here"
    );
    eprintln!("cmap fallback reads the ligatures as {text:?}");
}

const SHARED: &str = "<p style=\"font: 20px 'Noto Sans CJK TC'\">一⼀一</p>";

#[test]
fn a_shared_glyph_copies_as_the_code_point_typed() {
    let (_, text) = print(SHARED, Texts::Layout);
    assert_eq!(text, "一⼀一");
}

#[test]
fn the_cmap_fallback_prefers_the_ideograph_to_the_radical() {
    let (_, text) = print(SHARED, Texts::CmapOnly);
    assert_eq!(
        text, "一一一",
        "one glyph, read back as its canonical code point"
    );
}
