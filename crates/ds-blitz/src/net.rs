//! The net provider every quire document gets, one per kind of document: the app's own and its
//! frames' (`crate::frames` gives each frame document the frame one as it is parsed). Where a
//! request goes is `crate::route`'s table; this file only carries it out. `data:` and `file:`
//! are answered on the calling thread. The waker fires after each answer so the host paints the
//! frame the resource landed in: blitz applies a loaded image on the next resolve, one frame late
//! (spike S7).
//!
//! A frame's request for the app waits in the document's `FrameBook` until the frame is found
//! under its `iframe`, so it reaches the app with the frame's tag (`crate::frame_book`).

use crate::data_url;
use crate::frame_book::FrameBook;
use crate::frame_tag::FrameTag;
use crate::net_policy::{NetDecision, NetPolicy, NetReply, NetRequest};
use crate::origin::{FrameId, RequestOrigin};
use crate::route::{Document, Policy, Route, route};
use blitz_traits::net::{Bytes, NetHandler, NetProvider, NetWaker, Request};
use std::sync::Arc;

/// Serves one kind of document under the app's policy.
pub(crate) struct DsNet {
    /// The app's document or a frame's.
    served: Served,
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
            served: Served::Top,
            policy,
            fallback,
            waker,
        })
    }

    /// Every frame document's provider.
    pub(crate) fn frame(
        policy: NetPolicy,
        waker: Option<Arc<dyn NetWaker>>,
        book: FrameBook,
    ) -> Arc<dyn NetProvider> {
        Arc::new(DsNet {
            served: Served::Frame(book),
            policy,
            fallback: None,
            waker,
        })
    }

    /// Which kind of document this provider serves.
    fn document(&self) -> Document {
        match self.served {
            Served::Top => Document::Top,
            Served::Frame(_) => Document::Frame,
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

    /// Put the request to the app: the app document's now, a frame's once its tag is known.
    fn ask_app(&self, doc_id: usize, request: Request, handler: Box<dyn NetHandler>) {
        let NetPolicy::Custom(app) = &self.policy else {
            return;
        };
        let asking = Asking {
            app: Arc::clone(app),
            waker: self.waker.clone(),
            doc_id,
            request,
            handler,
        };
        match &self.served {
            Served::Top => asking.ask(RequestOrigin::Top, None),
            Served::Frame(book) => {
                let frame = FrameId::of(doc_id);
                book.ask(
                    frame,
                    Box::new(move |tag| asking.ask(RequestOrigin::Frame(frame), tag)),
                );
            }
        }
    }
}

/// Which document a provider serves, and for a frame, the book its requests wait in.
enum Served {
    Top,
    Frame(FrameBook),
}

/// One request on its way to the app.
struct Asking {
    app: Arc<dyn crate::net_policy::AppNet>,
    waker: Option<Arc<dyn NetWaker>>,
    doc_id: usize,
    request: Request,
    handler: Box<dyn NetHandler>,
}

impl Asking {
    /// Put the request to the app as coming from `origin` (tagged `tag`), and fetch it through
    /// the app if it is admitted.
    fn ask(self, origin: RequestOrigin, tag: Option<FrameTag>) {
        let asked = NetRequest::new(origin, tag, self.request.url);
        if self.app.decide(&asked) == NetDecision::Allow {
            let reply = NetReply::new(
                self.handler,
                asked.url().to_owned(),
                self.doc_id,
                self.waker,
            );
            self.app.fetch(asked, reply);
        }
    }
}

impl NetProvider for DsNet {
    fn fetch(&self, doc_id: usize, request: Request, handler: Box<dyn NetHandler>) {
        match route(
            self.document(),
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
