//! Printing a harness's document as it is now, for [`pdf_app`] and for tests that assert on a
//! printout's text rather than its pixels.

use crate::harness::Harness;
use crate::harness_config::HarnessConfig;
use crate::snapshot::{MOUNT_SETTLE, Viewport};
use blitz_dom::MediaType;
use dioxus::prelude::*;
use ds_blitz::{PageSpec, PdfError, content_size, print_document, print_viewport};

/// `app` as a PDF on `spec`'s pages: built as `config` says (its contexts and net policy; the
/// viewport is the page's content box), rendered until its mount-time work and images have
/// landed, as a snapshot is, then printed with [`Harness::pdf`].
pub fn pdf_app(
    app: fn() -> Element,
    config: HarnessConfig,
    spec: PageSpec,
) -> Result<Vec<u8>, PdfError> {
    let content = content_size(spec)?;
    let viewport = Viewport {
        width: content.width,
        height: content.height,
        scale_percent: 100,
    };
    let mut harness = Harness::with_config(app, config.with_viewport(viewport));
    harness.advance(MOUNT_SETTLE);
    harness.pdf(spec)
}

impl Harness {
    /// The document as a PDF on `spec`'s pages. For the printout it is laid out at the page's
    /// content width with `@media print` applied; afterwards its own viewport and media come
    /// back, so the harness carries on as before.
    pub fn pdf(&mut self, spec: PageSpec) -> Result<Vec<u8>, PdfError> {
        let print = print_viewport(spec)?;
        let (viewport, media) = {
            let mut doc = self.doc.doc.inner.borrow_mut();
            let kept = (doc.viewport().clone(), doc.media_type().clone());
            doc.set_viewport(print);
            doc.set_media_type(MediaType::print());
            kept
        };
        self.settle_now();
        let printed = print_document(&mut self.doc.doc.inner.borrow_mut(), spec);
        {
            let mut doc = self.doc.doc.inner.borrow_mut();
            doc.set_viewport(viewport);
            doc.set_media_type(media);
        }
        self.settle_now();
        printed
    }
}
