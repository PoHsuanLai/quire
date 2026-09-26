//! One window's document, built the way dioxus-native's `launch_cfg_with_props` builds it (with
//! quire's features: no `net`, no hot reload), so the first window and every window
//! `open_window` adds get the same document: `Host` around the root, the app's root contexts,
//! the net, HTML-parser and navigation providers dioxus-native would give it, and the quire faces.
//! dioxus-native adds its window contexts (the document, the window, `use_window_event`'s
//! registry, the shell, history, the renderer) when its application creates the window.

use crate::app_id::{AppId, with_app_id};
use crate::fonts::font_context;
use crate::host::{Host, HostProps};
use crate::native_providers::{AssetNet, LinkOpener};
use crate::setup::Setup;
use crate::window::Decorations;
use crate::window_requests::{Requests, Root};
use blitz_dom::HtmlParserProvider;
use blitz_shell::WindowConfig;
use blitz_traits::navigation::NavigationProvider;
use blitz_traits::net::NetProvider;
use dioxus::prelude::*;
use dioxus_native::winit::window::Window;
use dioxus_native::{DioxusNativeWindowRenderer, LogicalSize, WindowAttributes};
use dioxus_native_dom::{DioxusDocument, DocumentConfig};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

/// What every window of the app shares: what the app gave `launch`, and the request queue.
#[derive(Clone)]
pub(crate) struct Base {
    pub(crate) setup: Setup,
    pub(crate) app_id: Option<AppId>,
    pub(crate) decorations: Decorations,
    pub(crate) requests: Requests,
}

/// How one window starts.
#[derive(Debug, Clone)]
pub(crate) struct Shape {
    pub(crate) title: String,
    pub(crate) size: (u32, u32),
    pub(crate) app_id: Option<AppId>,
    pub(crate) decorations: Decorations,
}

/// The winit window a document ended up in, filled by its `Host` on the first render (which
/// dioxus-native runs as it creates the window), so the event loop can route that window's
/// events and raise it.
#[derive(Clone, Default)]
pub(crate) struct WindowSlot(Rc<RefCell<Option<Arc<dyn Window>>>>);

impl WindowSlot {
    /// Record the window.
    pub(crate) fn fill(&self, window: Arc<dyn Window>) {
        self.0.replace(Some(window));
    }

    /// The window, once created.
    pub(crate) fn window(&self) -> Option<Arc<dyn Window>> {
        self.0.borrow().clone()
    }
}

/// The window config for `root` shaped `shape`, drawn by `renderer`, reporting its window
/// through `slot`.
pub(crate) fn window_config(
    root: Root,
    shape: Shape,
    base: &Base,
    slot: &WindowSlot,
    renderer: DioxusNativeWindowRenderer,
) -> WindowConfig<DioxusNativeWindowRenderer> {
    let (width, height) = shape.size;
    let attributes = WindowAttributes::default()
        .with_title(shape.title)
        .with_surface_size(LogicalSize::new(width, height))
        .with_decorations(shape.decorations.winit());
    let attributes = match &shape.app_id {
        Some(id) => with_app_id(attributes, id),
        None => attributes,
    };
    let mut vdom = VirtualDom::new_with_props(Host, HostProps::new(root, base.setup.clone()));
    base.setup.contexts.install(&mut vdom);
    vdom.provide_root_context(base.requests.clone());
    vdom.provide_root_context(slot.clone());
    let net: Arc<dyn NetProvider> = Arc::new(AssetNet);
    vdom.provide_root_context(Arc::clone(&net));
    let parser: Arc<dyn HtmlParserProvider> = Arc::new(blitz_html::HtmlProvider);
    vdom.provide_root_context(Arc::clone(&parser));
    let navigation: Arc<dyn NavigationProvider> = Arc::new(LinkOpener);
    let doc = DioxusDocument::new(
        vdom,
        DocumentConfig {
            net_provider: Some(net),
            html_parser_provider: Some(parser),
            navigation_provider: Some(navigation),
            font_ctx: Some(font_context()),
            ..Default::default()
        },
    );
    WindowConfig::with_attributes(Box::new(doc) as _, renderer, attributes)
}
