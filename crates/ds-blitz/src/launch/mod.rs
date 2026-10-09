//! Running a quire app on Blitz: the document, the font registration, the input modality
//! (`ds::prelude::HostSignals`), a `data:` net provider for mask and background images (spike S7/S8),
//! and a redraw when a timer or an image lands.
//!
//! The window is blitz's portable `dioxus-native` shell (winit), so an app runs the same on any
//! OS; `crate::host` wraps the app to supply what quire needs on top of it. ds-blitz runs the
//! event loop itself (`crate::window_shell`), one dioxus-native application per window, so the
//! app can open more windows (`crate::open_window`).

mod runtime;

use crate::error::LaunchError;
pub use runtime::{RuntimeGuard, TokioSpawner, enter_runtime};

use crate::app_handle::AppHandle;
use crate::app_id::AppId;
use crate::app_life::{LastWindowClosed, Lifecycle};
use crate::click_focus::FocusFallback;
use crate::contexts::RootContexts;
use crate::frame_links::FrameLinks;
use crate::gpu_request::GpuRequest;
use crate::net_policy::NetPolicy;
use crate::open_window::WindowSpec;
use crate::setup::Setup;
use crate::startup_token::LaunchTokens;
use crate::window::Decorations;
use crate::window_build::Base;
use crate::window_requests::{Requests, Root};
use crate::window_shell::Windows;
use crate::window_size::WindowSize;
use dioxus::prelude::*;
use ds::window::icon::WindowIcon;
use std::time::Instant;
use tokio::runtime::Handle;

/// How the window starts, and what its document is given: build it with [`AppConfig::new`] and
/// the `with_*` methods.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// The first window: its title and size, and the application id and decorations every
    /// later window falls back to.
    first: WindowSpec,
    /// What the document is given beyond quire's own contexts.
    setup: Setup,
    /// What the app does when its last window closes.
    last_window: LastWindowClosed,
    /// The handle the app's own threads reach the loop through.
    handle: AppHandle,
    /// The wgpu features and limits the windows' device is created with.
    gpu: GpuRequest,
    /// The runtime the app's tasks run on; `None` is one `launch` builds and owns.
    runtime: Option<Handle>,
}

impl AppConfig {
    /// A first window of `size` titled `title`, with no app contexts.
    pub fn new(title: impl Into<String>, size: WindowSize) -> Self {
        AppConfig {
            first: WindowSpec::new(title, size),
            setup: Setup::default(),
            last_window: LastWindowClosed::default(),
            handle: AppHandle::new(),
            gpu: GpuRequest::default(),
            runtime: None,
        }
    }

    /// What the app does when its last window closes (default [`LastWindowClosed::Exit`]).
    /// Windows are independent: closing any of them, the first included, closes only that one.
    pub fn with_last_window(mut self, policy: LastWindowClosed) -> Self {
        self.last_window = policy;
        self
    }

    /// Bind `handle` to this app's event loop, so threads that hold a clone can open windows,
    /// redraw them or end the app (default: a handle of the app's own, reached from a window
    /// with [`use_app_handle`](crate::use_app_handle)).
    pub fn with_handle(mut self, handle: AppHandle) -> Self {
        self.handle = handle;
        self
    }

    /// The wgpu features and limits every window's device is requested with (default: the
    /// renderer's own, no extra features). A GPU that lacks one cannot make the device, so ask
    /// only for what the app cannot do without.
    pub fn with_gpu(mut self, gpu: GpuRequest) -> Self {
        self.gpu = gpu;
        self
    }

    /// Run the app's `tokio::spawn` calls on the runtime `handle` belongs to, the caller's own
    /// (default: `launch` builds a runtime of two worker threads and drops it when the loop ends).
    /// `launch` enters it on the calling thread for the whole run.
    pub fn with_runtime(mut self, handle: Handle) -> Self {
        self.runtime = Some(handle);
        self
    }

    /// The GPU request windows are opened with.
    pub fn gpu(&self) -> &GpuRequest {
        &self.gpu
    }

    /// The window's desktop application id (the Wayland `app_id`, the X11 `WM_CLASS`), so the
    /// desktop matches it to the app's `.desktop` file for its icon and name.
    pub fn with_app_id(mut self, id: AppId) -> Self {
        self.first = self.first.with_app_id(id);
        self
    }

    /// The icon every window shows, where the platform takes one from the window (X11 and Windows;
    /// winit has no window icon on macOS, where the bundle gives the dock icon). Wayland takes the icon from the app's `.desktop` file, matched by
    /// [`with_app_id`](AppConfig::with_app_id), so set that too. A window opened with its own
    /// [`WindowSpec::with_icon`] shows that one. The running window's icon changes with
    /// `ds::prelude::WindowHost::set_icon`.
    pub fn with_icon(mut self, icon: WindowIcon) -> Self {
        self.first = self.first.with_icon(icon);
        self
    }

    /// Who draws the window's frame (default [`Decorations::Server`]). An app whose root draws
    /// `ds::prelude::WindowFrame::Titlebar` passes [`Decorations::Client`], so the window has no second
    /// frame around it.
    pub fn with_decorations(mut self, decorations: Decorations) -> Self {
        self.first = self.first.with_decorations(decorations);
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

/// Run `app` in a first window until the app's [`LastWindowClosed`] policy ends the loop (by
/// default, when the last window closes). Windows it opens with [`crate::open_window`] or an
/// [`AppHandle`] are independent of the first: closing any one closes only it.
pub fn launch(app: fn() -> Element, config: AppConfig) -> Result<(), LaunchError> {
    run(Some(app), config)
}

/// Run the event loop with no window: the app opens its windows through the
/// [`AppHandle`] given to [`AppConfig::with_handle`] (the title and size of the `config` are
/// not used; its application id, decorations and contexts are every window's defaults). It
/// returns when the app's policy or `AppHandle::quit` ends the loop; with
/// [`LastWindowClosed::StayFor`] the loop's linger counts from the start.
pub fn launch_idle(config: AppConfig) -> Result<(), LaunchError> {
    run(None, config)
}

fn run(first: Option<fn() -> Element>, config: AppConfig) -> Result<(), LaunchError> {
    // First, before the runtime below or the event loop start a thread: clearing the activation
    // token from the environment while another thread may read it is undefined behaviour.
    let tokens = LaunchTokens::from_env();
    // Held until this call returns, which does not happen until the loop ends — i.e., for
    // the process's life. See `runtime` for why a host thread must enter Tokio at all.
    let (owned, given) = match config.runtime.clone() {
        Some(handle) => (None, handle),
        None => {
            let runtime = runtime::build()?;
            let handle = runtime.handle().clone();
            (Some(runtime), handle)
        }
    };
    let entered = given.enter();
    let event_loop = blitz_shell::create_default_event_loop();
    let waker = event_loop.create_proxy();
    config.handle.bind({
        let waker = event_loop.create_proxy();
        move || waker.wake_up()
    });
    let base = Base {
        setup: config.setup,
        app_id: config.first.app_id_or(None),
        icon: config.first.icon_or(None),
        decorations: config.first.decorations_or(Decorations::Server),
        requests: Requests::new(move || waker.wake_up()),
        handle: config.handle.clone(),
        gpu: config.gpu.clone(),
    };
    if let Some(app) = first {
        base.requests.open(config.first, Root::Plain(app));
    }
    let windows = Windows::new(
        base,
        Lifecycle::new(config.last_window, Instant::now()),
        tokens,
    );
    // An event loop that cannot run leaves the app no window to show anything in.
    let ran = event_loop.run_app(windows);
    config.handle.end();
    // Entered runtime first, then the owned one (dropping a runtime inside its own entry panics).
    drop(entered);
    drop(owned);
    ran.map_err(|error| LaunchError::EventLoop(Box::new(error)))
}
