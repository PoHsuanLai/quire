//! One HTML document for print: quire's faces, a network sealed to `data:` URLs (a printout
//! fetches nothing and reads no file), the HTML parser for `<iframe srcdoc>`, `@media print`
//! applied, laid out once at the page's content width.

use crate::fonts::font_context;
use crate::net::DsNet;
use crate::net_policy::NetPolicy;
use crate::pdf::spec::ContentBox;
use crate::wake::Wakeup;
use blitz_dom::{DocumentConfig, MediaType, StyleThreading};
use blitz_html::{HtmlDocument, HtmlProvider};
use blitz_traits::net::NetWaker;
use std::sync::Arc;

/// More resolves than this without the images settling is a document that keeps loading.
const MAX_ROUNDS: usize = 8;

/// `html` parsed and laid out into `content`, with every `data:` image it names landed.
pub(crate) fn document(html: &str, content: ContentBox) -> HtmlDocument {
    let wakeup = Arc::new(Wakeup::default());
    let fetches = Arc::clone(&wakeup);
    let waker: Arc<dyn NetWaker> = Arc::new(move |_doc: usize| fetches.note_fetch());
    let config = DocumentConfig {
        viewport: Some(content.viewport()),
        font_ctx: Some(font_context()),
        net_provider: Some(DsNet::top(NetPolicy::Sealed, None, Some(waker))),
        html_parser_provider: Some(Arc::new(HtmlProvider)),
        media_type: Some(MediaType::print()),
        style_threading: StyleThreading::Sequential,
        ..Default::default()
    };
    let mut doc = HtmlDocument::from_html(html, config);
    settle(&mut doc, &wakeup);
    doc
}

/// Resolve until a round lands no new resource: Blitz applies a loaded image on the resolve
/// after it lands (spike S7), and a `data:` image lands while the document is being built.
fn settle(doc: &mut HtmlDocument, wakeup: &Wakeup) {
    doc.resolve(0.0);
    for _ in 0..MAX_ROUNDS {
        let seen = wakeup.fetched();
        doc.resolve(0.0);
        if wakeup.fetched() == seen {
            return;
        }
    }
}
