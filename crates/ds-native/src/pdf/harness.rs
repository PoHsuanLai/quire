//! Printing a harness's document as it is now, for `pdf_app` and for tests that assert on a
//! printout's text rather than its pixels.

use crate::harness::Harness;
use crate::pdf::error::PdfError;
use crate::pdf::pages;
use crate::pdf::spec::{ContentBox, PageSpec};
use blitz_dom::MediaType;

impl Harness {
    /// The document as a PDF on `spec`'s pages. For the printout it is laid out at the page's
    /// content width with `@media print` applied; afterwards its own viewport and media come
    /// back, so the harness carries on as before.
    pub fn pdf(&mut self, spec: PageSpec) -> Result<Vec<u8>, PdfError> {
        let content = ContentBox::of(spec)?;
        let (viewport, media) = {
            let mut doc = self.doc.doc.inner.borrow_mut();
            let kept = (doc.viewport().clone(), doc.media_type().clone());
            doc.set_viewport(content.viewport());
            doc.set_media_type(MediaType::print());
            kept
        };
        self.settle_now();
        let printed = pages::print(&mut self.doc.doc.inner.borrow_mut(), content);
        {
            let mut doc = self.doc.doc.inner.borrow_mut();
            doc.set_viewport(viewport);
            doc.set_media_type(media);
        }
        self.settle_now();
        printed.map(|printed| printed.bytes)
    }
}
