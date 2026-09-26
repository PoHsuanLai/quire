//! How a [`Harness`](crate::Harness) builds its document: the viewport, and the same providers
//! an [`AppConfig`](crate::AppConfig) gives a window, so a test sees what the window would.

use crate::click_focus::FocusFallback;
use crate::contexts::RootContexts;
use crate::frame_links::FrameLinks;
use crate::gpu_adapter::AdapterPref;
use crate::harness_backend::Backend;
use crate::harness_clock::Clock;
use crate::net_policy::NetPolicy;
use crate::setup::Setup;
use crate::snapshot::Viewport;

/// A headless document's size and providers: build it with [`HarnessConfig::new`] and the
/// `with_*` methods, then pass it to [`Harness::with_config`](crate::Harness::with_config) or
/// [`snapshot_with`](crate::snapshot::snapshot_with).
#[derive(Debug, Clone)]
pub struct HarnessConfig {
    viewport: Viewport,
    setup: Setup,
    backend: Backend,
    adapter: AdapterPref,
    clock: Clock,
}

impl HarnessConfig {
    /// A document at `viewport` with no app contexts.
    pub fn new(viewport: Viewport) -> Self {
        HarnessConfig {
            viewport,
            setup: Setup::default(),
            backend: Backend::default(),
            adapter: AdapterPref::default(),
            clock: Clock::default(),
        }
    }

    /// Provide `value` at the root, read with `use_context::<T>()`.
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

    /// The renderer pictures and [`Harness::paint_timed`](crate::Harness::paint_timed) paint
    /// with (default [`Backend::Cpu`]). See [`Backend::Hybrid`] for how a missing GPU shows.
    pub fn with_backend(mut self, backend: Backend) -> Self {
        self.backend = backend;
        self
    }

    /// Which GPU a [`Backend::Hybrid`] harness opens (default [`AdapterPref::Auto`]); a
    /// non-empty `WGPU_ADAPTER_NAME` overrides it.
    pub fn with_adapter(mut self, adapter: AdapterPref) -> Self {
        self.adapter = adapter;
        self
    }

    /// The clock the harness's timers run on (default [`Clock::Wall`]). [`Clock::Virtual`]
    /// makes `advance` move one clock for CSS animations and every ds timer, so a test sees the
    /// same frames however loaded the machine is (sill Q380).
    pub fn with_clock(mut self, clock: Clock) -> Self {
        self.clock = clock;
        self
    }

    /// The clock the harness's timers run on.
    pub fn clock(&self) -> Clock {
        self.clock
    }

    /// The renderer the document paints with.
    pub fn backend(&self) -> Backend {
        self.backend
    }

    /// The GPU a hybrid harness prefers.
    pub fn adapter(&self) -> &AdapterPref {
        &self.adapter
    }

    /// The same providers at `viewport`.
    pub(crate) fn with_viewport(mut self, viewport: Viewport) -> Self {
        self.viewport = viewport;
        self
    }

    /// The size and scale the document renders at.
    pub fn viewport(&self) -> Viewport {
        self.viewport
    }

    pub(crate) fn setup(&self) -> &Setup {
        &self.setup
    }
}
