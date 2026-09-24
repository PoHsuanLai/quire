//! The net provider every quire document gets, one per kind of document: the app's own and its
//! frames' (`crate::frames` gives each frame document the frame one as it is parsed). Where a
//! request goes is `crate::route`'s table; this file only carries it out. `data:` and `file:`
//! are answered on the calling thread. The waker fires after each answer so the host paints the
//! frame the resource landed in: blitz applies a loaded image on the next resolve, one frame late
//! (spike S7).

use crate::data_url;
use crate::net_policy::{NetPolicy, NetReply, NetRequest};
use crate::origin::{FrameId, RequestOrigin};
use crate::route::{Document, Policy, Route, route};
use blitz_traits::net::{Bytes, NetHandler, NetProvider, NetWaker, Request};
use std::sync::Arc;

/// Serves one kind of document under the app's policy.
pub(crate) struct DsNet {
    /// The app's document or a frame's.
    document: Document,
    /// The app's choice.
    policy: NetPolicy,
    /// Who answers the app document's other schemes under `NetPolicy::Local`, if anyone.
    fallback: Option<Arc<dyn NetProvider>>,
    /// Told when a resource has been handed to the document.
    waker: Option<Arc<dyn NetWaker>>,
}

impl DsNet {
    /// The app document's provider: under `Local`, other schemes go to `fallback`.
    pub(crate) fn top(
        policy: NetPolicy,
        fallback: Option<Arc<dyn NetProvider>>,
        waker: Option<Arc<dyn NetWaker>>,
    ) -> Arc<dyn NetProvider> {
        Arc::new(DsNet {
            document: Document::Top,
            policy,
            fallback,
            waker,
        })
    }

    /// Every frame document's provider.
    pub(crate) fn frame(
        policy: NetPolicy,
        waker: Option<Arc<dyn NetWaker>>,
    ) -> Arc<dyn NetProvider> {
        Arc::new(DsNet {
            document: Document::Frame,
            policy,
            fallback: None,
            waker,
        })
    }

    /// Where a request from Blitz document `doc_id` came from.
    fn origin(&self, doc_id: usize) -> RequestOrigin {
        match self.document {
            Document::Top => RequestOrigin::Top,
            Document::Frame => RequestOrigin::Frame(FrameId::of(doc_id)),
        }
    }

    /// Hand `bytes` (if any) to the document and wake its host. A malformed data: URL or an
    /// unreadable file is dropped, as a browser shows a broken image: the handler has no error
    /// path.
    fn answer(
        &self,
        doc_id: usize,
        url: String,
        bytes: Option<Vec<u8>>,
        handler: Box<dyn NetHandler>,
    ) {
        if let Some(bytes) = bytes {
            handler.bytes(url, Bytes::from(bytes));
            if let Some(waker) = &self.waker {
                waker.wake(doc_id);
            }
        }
    }

    /// Put the request to the app, and fetch it through the app if it is admitted.
    fn ask_app(&self, doc_id: usize, request: Request, handler: Box<dyn NetHandler>) {
        let NetPolicy::Custom(app) = &self.policy else {
            return;
        };
        let asked = NetRequest::new(self.origin(doc_id), request.url);
        if app.decide(&asked) == crate::net_policy::NetDecision::Allow {
            let reply = NetReply::new(handler, asked.url().to_owned(), doc_id, self.waker.clone());
            app.fetch(asked, reply);
        }
    }
}

impl NetProvider for DsNet {
    fn fetch(&self, doc_id: usize, request: Request, handler: Box<dyn NetHandler>) {
        match route(
            self.document,
            request.url.scheme(),
            Policy::of(&self.policy),
        ) {
            Route::Data => {
                let bytes = data_url::decode(request.url.as_str());
                self.answer(doc_id, request.url.to_string(), bytes, handler);
            }
            Route::File => {
                let bytes = request
                    .url
                    .to_file_path()
                    .ok()
                    .and_then(|path| std::fs::read(path).ok());
                self.answer(doc_id, request.url.to_string(), bytes, handler);
            }
            Route::Fallback => {
                if let Some(fallback) = &self.fallback {
                    fallback.fetch(doc_id, request, handler);
                }
            }
            Route::App => self.ask_app(doc_id, request, handler),
            Route::Drop => {}
        }
    }
}
