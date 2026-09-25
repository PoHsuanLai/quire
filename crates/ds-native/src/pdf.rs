//! PDF output: a quire document laid out by Blitz, paginated by quire's rules, and painted into
//! a vector PDF through the krilla painter (`anyrender_krilla`). Text is real text in embedded,
//! subsetted faces (selectable, searchable, taken from the layout rather than guessed from the
//! font); a JPEG is embedded as it arrived.
//!
//! The document is built as a snapshot's is (quire's faces, the HTML parser, sequential
//! styling) with `@media print` applied, laid out once at the content box's width at scale 1,
//! and cut into pages by `paginate`'s rules, which read markers rather than CSS (Blitz drops
//! `break-*` and `@page`): `data-break-before="page"` and `data-break-inside="avoid"`.

mod error;
mod flow;
mod harness;
mod html;
mod images;
mod pages;
mod paginate;
mod run_texts;
mod spec;
#[cfg(test)]
mod text_tests;

pub use error::PdfError;
pub use spec::{Margins, PageSize, PageSpec, Pt};

use crate::harness::Harness;
use crate::harness_config::HarnessConfig;
use crate::snapshot::MOUNT_SETTLE;
use dioxus::prelude::*;
use spec::ContentBox;

/// `html` (a whole document) as a PDF on `spec`'s pages. The network is sealed: only `data:`
/// URLs load, so a printout fetches nothing and reads no file.
pub fn pdf(html: &str, spec: PageSpec) -> Result<Vec<u8>, PdfError> {
    let content = ContentBox::of(spec)?;
    let mut doc = html::document(html, content);
    pages::print(&mut doc, content).map(|printed| printed.bytes)
}

/// `app` as a PDF on `spec`'s pages: built as `config` says (its contexts and net policy; the
/// viewport is the page's content box), rendered until its mount-time work and images have
/// landed, as a snapshot is, then printed with [`Harness::pdf`].
pub fn pdf_app(
    app: fn() -> Element,
    config: HarnessConfig,
    spec: PageSpec,
) -> Result<Vec<u8>, PdfError> {
    let content = ContentBox::of(spec)?;
    let mut harness = Harness::with_config(app, config.with_viewport(content.logical()));
    harness.advance(MOUNT_SETTLE);
    harness.pdf(spec)
}
