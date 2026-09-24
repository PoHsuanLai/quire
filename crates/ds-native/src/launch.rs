//! Running a quire app on Blitz: the document, the font registration, the input modality
//! (`ds::HostModality`), a `data:` net provider for mask and background images (spike S7/S8),
//! and a redraw when a timer or an image lands.
//!
//! The window is blitz's portable `dioxus-native` shell (winit), so an app runs the same on any
//! OS; `crate::host` wraps the app to supply what quire needs on top of it.

use crate::contexts::RootContexts;
use crate::fonts::font_context;
use crate::host::{Host, HostProps};
use crate::net_policy::NetPolicy;
use crate::setup::Setup;
use dioxus::prelude::*;
use dioxus_native::{LogicalSize, WindowAttributes};

/// How the window starts, and what its document is given: build it with [`AppConfig::new`] and
/// the `with_*` methods.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// The window title.
    title: String,
    /// The initial width in logical pixels.
    width: u32,
    /// The initial height in logical pixels.
    height: u32,
    /// What the document is given beyond quire's own contexts.
    setup: Setup,
}

impl AppConfig {
    /// A `width` x `height` window (logical pixels) titled `title`, with no app contexts.
    pub fn new(title: impl Into<String>, width: u32, height: u32) -> Self {
        AppConfig {
            title: title.into(),
            width,
            height,
            setup: Setup::default(),
        }
    }

    /// Provide `value` at the root, read with `use_context::<T>()` anywhere in the app: the
    /// window's equivalent of dioxus desktop's `LaunchBuilder::with_context`.
    pub fn with_context<T: Clone + Send + Sync + 'static>(mut self, value: T) -> Self {
        self.setup.contexts = self.setup.contexts.with(value);
        self
    }

    /// Who answers the document's requests beyond `data:`, and its frames' (default
    /// [`NetPolicy::Local`]).
    pub fn with_net(mut self, policy: NetPolicy) -> Self {
        self.setup.net = policy;
        self
    }

    /// Provide every value in `contexts` at the root, after any already given.
    pub fn with_contexts(mut self, contexts: RootContexts) -> Self {
        self.setup.contexts = self.setup.contexts.and(contexts);
        self
    }
}

/// Run `app` until its window closes.
pub fn launch(app: fn() -> Element, config: AppConfig) {
    // Held until this call returns, which does not happen until the window closes — i.e., for
    // the process's life. See `crate::runtime` for why a host thread must enter Tokio at all.
    let _runtime = crate::runtime::enter();
    let window = WindowAttributes::default()
        .with_title(config.title)
        .with_surface_size(LogicalSize::new(config.width, config.height));
    let native = dioxus_native::Config::new()
        .with_window_attributes(window)
        .with_font_ctx(font_context());
    let contexts = config.setup.contexts.for_launch();
    dioxus_native::launch_cfg_with_props(
        Host,
        HostProps::new(app, config.setup),
        contexts,
        vec![Box::new(native)],
    );
}
