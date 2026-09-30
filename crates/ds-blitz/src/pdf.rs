//! PDF output: a quire document laid out by Blitz, paginated by quire's rules, and painted into
//! a vector PDF through the pdfrum painter (`pdfrum-anyrender`). Text is real text in embedded,
//! subsetted faces (selectable, searchable, taken from the layout rather than guessed from the
//! font); a JPEG is embedded as it arrived.
//!
//! [`content_size`], [`print_viewport`] and [`print_document`] let `ds-harness` print a
//! harness's document.
//!
//! The document is built as a snapshot's is (quire's faces, the HTML parser, sequential
//! styling) with `@media print` applied, laid out once at the content box's width at scale 1,
//! and cut into pages by `paginate`'s rules, which read markers rather than CSS (Blitz drops
//! `break-*` and `@page`): `data-break-before="page"` and `data-break-inside="avoid"`.

mod error;
mod flow;
mod html;
mod images;
mod pages;
mod paginate;
mod run_texts;
mod spec;
#[cfg(test)]
mod text_tests;

pub use error::PdfError;
pub use spec::{ContentPx, Margins, PageSize, PageSpec, Pt};

use blitz_dom::BaseDocument;
use blitz_traits::shell::Viewport;
use spec::ContentBox;

/// `html` (a whole document) as a PDF on `spec`'s pages. The network is sealed: only `data:`
/// URLs load, so a printout fetches nothing and reads no file.
pub fn pdf(html: &str, spec: PageSpec) -> Result<Vec<u8>, PdfError> {
    let content = ContentBox::of(spec)?;
    let mut doc = html::document(html, content);
    pages::print(&mut doc, content).map(|printed| printed.bytes)
}

/// The logical size of `spec`'s content box: the viewport a document is laid out at to be printed.
pub fn content_size(spec: PageSpec) -> Result<ContentPx, PdfError> {
    Ok(ContentBox::of(spec)?.logical())
}

/// The viewport `spec`'s content box asks of a document that is laid out to be printed.
pub fn print_viewport(spec: PageSpec) -> Result<Viewport, PdfError> {
    Ok(ContentBox::of(spec)?.viewport())
}

/// `doc`, already laid out on [`print_viewport`] with `@media print` applied, as a PDF on
/// `spec`'s pages.
pub fn print_document(doc: &mut BaseDocument, spec: PageSpec) -> Result<Vec<u8>, PdfError> {
    let content = ContentBox::of(spec)?;
    pages::print(doc, content).map(|printed| printed.bytes)
}
