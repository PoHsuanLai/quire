//! What the app decides about the network (mailo Phase B, G3): whether ds-native serves only
//! local resources, nothing but inline ones, or hands the rest to the app's own fetcher (mail's
//! consented remote images). Frames never get `file:` from ds-native: a frame's markup is not
//! the app's, so a sanitiser miss must not turn into a read of the disk.

use crate::origin::RequestOrigin;
use blitz_traits::net::{Bytes, NetHandler, NetWaker, Url};
use std::fmt;
use std::sync::Arc;

/// Who answers a document's requests beyond inline `data:` (always served, to every document).
#[derive(Clone, Default)]
pub enum NetPolicy {
    /// The app's document also gets `file:`, and any other scheme goes to dioxus-native's own
    /// provider (the `dioxus:` asset scheme). Frames get `data:` only. What `launch` always did.
    #[default]
    Local,
    /// As `Local` for the app's own `file:`; every other request, from the app's document or a
    /// frame, `file:` from a frame included, is put to the app's [`AppNet`], which admits or
    /// refuses it and fetches what it admits.
    Custom(Arc<dyn AppNet>),
    /// `data:` only, for every document: nothing leaves the process or reads the disk.
    Sealed,
}

impl fmt::Debug for NetPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            NetPolicy::Local => "Local",
            NetPolicy::Custom(_) => "Custom(..)",
            NetPolicy::Sealed => "Sealed",
        })
    }
}

/// The app's fetcher: a real seam, swapped between the app's HTTP client and a test's recorder.
pub trait AppNet: Send + Sync + 'static {
    /// Whether `request` may be fetched. Called on the UI thread while the document is held, so
    /// it decides from the request alone (its origin, its URL) and returns at once.
    fn decide(&self, request: &NetRequest) -> NetDecision;

    /// Fetch an admitted `request` and hand the bytes to `reply`, from any thread (a spawned
    /// task, once the response lands). Dropping `reply` unanswered leaves the resource missing,
    /// as a browser shows a broken image.
    fn fetch(&self, request: NetRequest, reply: NetReply);
}

/// The app's answer for one request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NetDecision {
    /// Fetch it: [`AppNet::fetch`] is called next.
    Allow,
    /// Drop it: the document never gets the resource.
    Deny,
}

/// One request a document made, and where it came from.
#[derive(Debug, Clone)]
pub struct NetRequest {
    origin: RequestOrigin,
    url: Url,
}

impl NetRequest {
    pub(crate) fn new(origin: RequestOrigin, url: Url) -> Self {
        NetRequest { origin, url }
    }

    /// The document that asked.
    pub fn origin(&self) -> RequestOrigin {
        self.origin
    }

    /// The absolute URL asked for.
    pub fn url(&self) -> &str {
        self.url.as_str()
    }

    /// The URL's scheme, lower-case: `https`, `cid`, `file`.
    pub fn scheme(&self) -> &str {
        self.url.scheme()
    }
}

/// Where the app delivers what it fetched: the asking document, and the waker that makes its
/// host paint the frame the resource lands in.
pub struct NetReply {
    handler: Box<dyn NetHandler>,
    url: String,
    document: usize,
    waker: Option<Arc<dyn NetWaker>>,
}

impl NetReply {
    pub(crate) fn new(
        handler: Box<dyn NetHandler>,
        url: String,
        document: usize,
        waker: Option<Arc<dyn NetWaker>>,
    ) -> Self {
        NetReply {
            handler,
            url,
            document,
            waker,
        }
    }

    /// Hand `bytes` to the document as the requested resource.
    pub fn bytes(self, bytes: Vec<u8>) {
        self.handler.bytes(self.url, Bytes::from(bytes));
        if let Some(waker) = &self.waker {
            waker.wake(self.document);
        }
    }
}

impl fmt::Debug for NetReply {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NetReply").field("url", &self.url).finish()
    }
}
