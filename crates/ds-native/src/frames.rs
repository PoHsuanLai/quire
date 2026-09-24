//! Frames (`<iframe>` sub-documents) get their own providers from birth. Blitz builds a frame's
//! document by handing the parent's parser a `DocumentConfig` that inherits the parent's net
//! provider (`blitz-dom/src/iframe.rs`), so a mail body inside a frame would be served `file:`
//! like the app. Wrapping the parser is the one place that sees every frame document before it
//! exists (a `srcdoc`, a `src` load, a frame nested in a frame): it swaps the inherited provider
//! for the frame one before the document parses, and so before its first image is asked for.

use blitz_dom::{Document, DocumentConfig, DocumentMutator, HtmlParserProvider, NodeId};
use blitz_traits::net::NetProvider;
use std::sync::Arc;

/// The document's parser, with every frame document it builds given `net`.
pub(crate) struct FrameParser {
    /// The parser that does the work.
    inner: Arc<dyn HtmlParserProvider>,
    /// What every frame document fetches through.
    net: Arc<dyn NetProvider>,
}

impl FrameParser {
    /// `inner`, giving each frame document it builds `net`.
    pub(crate) fn shared(
        inner: Arc<dyn HtmlParserProvider>,
        net: Arc<dyn NetProvider>,
    ) -> Arc<dyn HtmlParserProvider> {
        // Blitz's `DocumentConfig` holds the parser as an `Arc` though parsers are not `Send`;
        // it never leaves the document's thread.
        #[allow(clippy::arc_with_non_send_sync)]
        let parser = Arc::new(FrameParser { inner, net });
        parser
    }
}

impl HtmlParserProvider for FrameParser {
    fn parse_inner_html<'m, 'doc>(
        &self,
        mutr: &'m mut DocumentMutator<'doc>,
        element_id: NodeId,
        html: &str,
    ) {
        self.inner.parse_inner_html(mutr, element_id, html);
    }

    fn parse_document(&self, html: &str, mut config: DocumentConfig) -> Box<dyn Document> {
        // The config's parser is this one (the frame inherits its parent's), so a frame nested
        // in this frame comes back here and is sealed the same way.
        config.net_provider = Some(Arc::clone(&self.net));
        self.inner.parse_document(html, config)
    }
}
