//! The two providers dioxus-native gives a window's document and keeps private, rebuilt here
//! because ds-native now builds the document itself (`crate::window_build`) so a running app can
//! open a second window. With dioxus-native's features as quire sets them (no `net`, no
//! `data-uri`), its net provider serves the `dioxus:` asset scheme and nothing else, and its
//! navigation provider hands a clicked `http:`, `https:` or `mailto:` link to the desktop's
//! browser. These do the same.
//!
//! ds-native's own net policy wraps [`AssetNet`] once the document mounts (`crate::install`):
//! under `NetPolicy::Local` it is the fallback for any scheme ds-native does not serve itself.

use blitz_traits::navigation::{NavigationOptions, NavigationProvider};
use blitz_traits::net::{Method, NetHandler, NetProvider, Request};

/// The `dioxus:` asset scheme (an `asset!()` path, bundled by the dioxus CLI).
pub(crate) struct AssetNet;

impl NetProvider for AssetNet {
    fn fetch(&self, _doc_id: usize, request: Request, handler: Box<dyn NetHandler>) {
        if request.url.scheme() != "dioxus" {
            return;
        }
        // A missing asset is left unanswered, as a missing resource is anywhere else.
        if let Ok(response) = dioxus_asset_resolver::native::serve_asset(request.url.path()) {
            handler.bytes(request.url.to_string(), response.into_body().into());
        }
    }
}

/// A top-level link the document navigates to: the desktop's browser (or mail client) opens it.
pub(crate) struct LinkOpener;

impl NavigationProvider for LinkOpener {
    fn navigate_to(&self, options: NavigationOptions) {
        if options.method == Method::GET
            && matches!(options.url.scheme(), "http" | "https" | "mailto")
        {
            // Nothing in the window can show that no browser is set up.
            let _ = webbrowser::open(options.url.as_str());
        }
    }
}
