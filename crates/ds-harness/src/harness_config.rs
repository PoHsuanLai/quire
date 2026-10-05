//! How a [`Harness`](crate::Harness) builds its document: the viewport, and the same providers
//! an [`AppConfig`](crate::AppConfig) gives a window, so a test sees what the window would.

use crate::gpu_diagnostics::GpuDiagnostics;
use crate::harness_backend::Backend;
use crate::harness_clock::Clock;
use crate::headless::Layout;
use crate::snapshot::Viewport;
use ds_blitz::AdapterPref;
use ds_blitz::FocusFallback;
use ds_blitz::FrameLinks;
use ds_blitz::NetPolicy;
use ds_blitz::RootContexts;
use ds_blitz::seam::Setup;

/// A headless document's size and providers: build it with [`HarnessConfig::new`] and the
/// `with_*` methods, then pass it to [`Harness::new`](crate::Harness::new) or
/// [`snapshot_with`](crate::snapshot::snapshot_with).
#[derive(Debug, Clone)]
pub struct HarnessConfig {
    viewport: Viewport,
    setup: Setup,
    backend: Backend,
    adapter: AdapterPref,
    diagnostics: GpuDiagnostics,
    clock: Clock,
    layout: Layout,
}

impl HarnessConfig {
    /// A document at `viewport` with no app contexts.
    pub fn new(viewport: Viewport) -> Self {
        HarnessConfig {
            viewport,
            setup: Setup::default(),
            backend: Backend::default(),
            adapter: AdapterPref::default(),
            diagnostics: GpuDiagnostics::default(),
            clock: Clock::default(),
            layout: Layout::default(),
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

    /// Whether a [`Backend::Hybrid`] harness's GPU instance names its objects and validates
    /// (default [`GpuDiagnostics::Off`], whatever `WGPU_DEBUG` and `WGPU_VALIDATION` say). Turn
    /// it on for one debugging session; see [`GpuDiagnostics`] for why a suite should not.
    pub fn with_gpu_diagnostics(mut self, diagnostics: GpuDiagnostics) -> Self {
        self.diagnostics = diagnostics;
        self
    }

    /// The clock the harness's timers run on (default [`Clock::Wall`]). [`Clock::Virtual`]
    /// makes `advance` move one clock for CSS animations and every ds timer, so a test sees the
    /// same frames however loaded the machine is.
    pub fn with_clock(mut self, clock: Clock) -> Self {
        self.clock = clock;
        self
    }

    /// Whether the document is laid out as it renders (default [`Layout::Running`]).
    /// [`Layout::Held`] builds it as a shell surface is built before it is mapped: its renders
    /// run and its tasks are polled, but nothing is styled or laid out until
    /// [`Harness::map`](crate::Harness::map) (every rect reads 0 x 0 until then).
    pub fn with_layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    /// The clock the harness's timers run on.
    pub fn clock(&self) -> Clock {
        self.clock
    }

    /// Whether the document is laid out as it renders.
    pub fn layout(&self) -> Layout {
        self.layout
    }

    /// The renderer the document paints with.
    pub fn backend(&self) -> Backend {
        self.backend
    }

    /// Whether a hybrid harness's GPU instance names objects and validates.
    pub fn gpu_diagnostics(&self) -> GpuDiagnostics {
        self.diagnostics
    }

    /// The GPU a hybrid harness prefers.
    pub fn adapter(&self) -> &AdapterPref {
        &self.adapter
    }

    /// The same providers at `viewport`.
    #[cfg(feature = "pdf")]
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

impl From<Viewport> for HarnessConfig {
    fn from(viewport: Viewport) -> Self {
        HarnessConfig::new(viewport)
    }
}
