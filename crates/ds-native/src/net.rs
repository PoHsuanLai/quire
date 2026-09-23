//! The net provider every quire document gets: `data:` URLs (the grain PNG and icon masks, spike
//! S7/S8) and `file:` URLs, answered on the calling thread; anything else goes to the provider
//! the document already had, if any. The waker fires after each answer so the host paints the
//! frame the resource landed in: blitz applies a loaded image on the next resolve, one frame late
//! (spike S7).

use crate::data_url;
use blitz_traits::net::{Bytes, NetHandler, NetProvider, NetWaker, Request};
use std::sync::Arc;

/// Serves `data:` and `file:`, delegating other schemes.
pub(crate) struct DsNet {
    /// Who answers the schemes this provider does not.
    fallback: Option<Arc<dyn NetProvider>>,
    /// Told when a resource has been handed to the document.
    waker: Option<Arc<dyn NetWaker>>,
}

impl DsNet {
    /// A provider that wakes `waker` after every answer and hands other schemes to `fallback`.
    pub(crate) fn shared(
        fallback: Option<Arc<dyn NetProvider>>,
        waker: Option<Arc<dyn NetWaker>>,
    ) -> Arc<dyn NetProvider> {
        Arc::new(DsNet { fallback, waker })
    }
}

impl NetProvider for DsNet {
    fn fetch(&self, doc_id: usize, request: Request, handler: Box<dyn NetHandler>) {
        let bytes = match request.url.scheme() {
            "data" => data_url::decode(request.url.as_str()),
            "file" => request
                .url
                .to_file_path()
                .ok()
                .and_then(|path| std::fs::read(path).ok()),
            _ => {
                if let Some(fallback) = &self.fallback {
                    fallback.fetch(doc_id, request, handler);
                }
                return;
            }
        };
        // A malformed data: URL or an unreadable file is dropped, as a browser shows a broken
        // image: the handler has no error path.
        if let Some(bytes) = bytes {
            handler.bytes(request.url.to_string(), Bytes::from(bytes));
            if let Some(waker) = &self.waker {
                waker.wake(doc_id);
            }
        }
    }
}
