//! Putting the app's providers on a window's document once it exists. dioxus-native builds the
//! document itself and hands the app nothing that can see it, so `crate::host` reaches it
//! through a hidden element's `onmounted` handle and calls [`install`] there, before the app's
//! first render.

use crate::clipboard::HostClipboard;
use crate::frame_book::FrameBook;
use crate::frames::FrameParser;
use crate::net::DsNet;
use crate::setup::Setup;
use blitz_traits::navigation::NavigationProvider;
use blitz_traits::net::NetWaker;
use dioxus_native_dom::NodeHandle;
use std::sync::Arc;

/// Put `setup`'s net policy on the document and on every frame it will build, waking the
/// window's shell to paint when a resource lands; give every frame `frame_nav` and hold its
/// requests in `book` until it is found; and reach the clipboard through the shell.
pub(crate) fn install(
    handle: &NodeHandle,
    setup: &Setup,
    clipboard: &HostClipboard,
    frames: (Arc<dyn NavigationProvider>, FrameBook),
) {
    let (frame_nav, book) = frames;
    let mut doc = handle.doc_mut();
    clipboard.set(Arc::clone(&doc.shell_provider));
    let shell = Arc::clone(&doc.shell_provider);
    let waker: Arc<dyn NetWaker> = Arc::new(move |_doc: usize| shell.request_redraw());
    let fallback = Arc::clone(&doc.net_provider);
    let parser = Arc::clone(&doc.html_parser_provider);
    let frame_net = DsNet::frame(setup.net.clone(), Some(Arc::clone(&waker)), book);
    doc.set_net_provider(DsNet::top(setup.net.clone(), Some(fallback), Some(waker)));
    doc.set_html_parser_provider(FrameParser::shared(parser, frame_net, frame_nav));
}
