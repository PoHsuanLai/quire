//! Running a quire app on Blitz: the document, the font registration, the input modality
//! (`ds::HostModality`), a `data:` net provider for mask and background images (spike S7/S8),
//! and a redraw when a timer or an image lands.
//!
//! The window is blitz's portable `dioxus-native` shell (winit), so an app runs the same on any
//! OS; `crate::host` wraps the app to supply what quire needs on top of it. ds-native runs the
//! event loop itself (`crate::window_shell`), one dioxus-native application per window, so the
//! app can open more windows (`crate::open_window`).

use crate::app_id::AppId;
use crate::click_focus::FocusFallback;
use crate::contexts::RootContexts;
use crate::frame_links::FrameLinks;
use crate::net_policy::NetPolicy;
use crate::setup::Setup;
use crate::window::Decorations;
use crate::window_build::{Base, Shape};
use crate::window_requests::{Requests, Root};
use crate::window_shell::Windows;
use dioxus::prelude::*;

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
    /// The desktop application id, if the app has one.
    app_id: Option<AppId>,
    /// Who draws the window's frame.
    decorations: Decorations,
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
            app_id: None,
            decorations: Decorations::Server,
            setup: Setup::default(),
        }
    }

    /// The window's desktop application id (the Wayland `app_id`, the X11 `WM_CLASS`), so the
    /// desktop matches it to the app's `.desktop` file for its icon and name.
    pub fn with_app_id(mut self, id: AppId) -> Self {
        self.app_id = Some(id);
        self
    }

    /// Who draws the window's frame (default [`Decorations::Server`]). An app whose root draws
    /// `ds::WindowFrame::Titlebar` passes [`Decorations::Client`], so the window has no second
    /// frame around it.
    pub fn with_decorations(mut self, decorations: Decorations) -> Self {
        self.decorations = decorations;
        self
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

    /// What a link clicked inside a frame does (default [`FrameLinks::Inert`]): a frame never
    /// navigates, so the app opens the link itself or nothing happens.
    pub fn with_frame_links(mut self, links: FrameLinks) -> Self {
        self.setup.frame_links = links;
        self
    }

    /// Where the keyboard goes after a click on nothing focusable (default
    /// [`FocusFallback::Ancestor`]: the nearest focusable ancestor, as in a browser).
    pub fn with_focus_fallback(mut self, fallback: FocusFallback) -> Self {
        self.setup.focus_fallback = fallback;
        self
    }

    /// Provide every value in `contexts` at the root, after any already given.
    pub fn with_contexts(mut self, contexts: RootContexts) -> Self {
        self.setup.contexts = self.setup.contexts.and(contexts);
        self
    }
}

/// Run `app` until its window closes. Windows it opens with [`crate::open_window`] close with
/// it.
pub fn launch(app: fn() -> Element, config: AppConfig) {
    // Held until this call returns, which does not happen until the window closes — i.e., for
    // the process's life. See `crate::runtime` for why a host thread must enter Tokio at all.
    let _runtime = crate::runtime::enter();
    let event_loop = blitz_shell::create_default_event_loop();
    let waker = event_loop.create_proxy();
    let base = Base {
        setup: config.setup,
        app_id: config.app_id.clone(),
        decorations: config.decorations,
        requests: Requests::new(move || waker.wake_up()),
    };
    let shape = Shape {
        title: config.title,
        size: (config.width, config.height),
        app_id: config.app_id,
        decorations: config.decorations,
    };
    let windows = Windows::new(event_loop.create_proxy(), Root::Plain(app), shape, base);
    // As dioxus-native's own `launch` does: an event loop that cannot run leaves the app no
    // window to show anything in, which is a broken host, not bad input.
    event_loop
        .run_app(windows)
        .expect("the window's event loop could not run");
}
