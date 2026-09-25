//! How a [`Harness`](crate::Harness) builds its document: the viewport, and the same providers
//! an [`AppConfig`](crate::AppConfig) gives a window, so a test sees what the window would.

use crate::click_focus::FocusFallback;
use crate::contexts::RootContexts;
use crate::frame_links::FrameLinks;
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
}

impl HarnessConfig {
    /// A document at `viewport` with no app contexts.
    pub fn new(viewport: Viewport) -> Self {
        HarnessConfig {
            viewport,
            setup: Setup::default(),
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

    /// The size and scale the document renders at.
    pub fn viewport(&self) -> Viewport {
        self.viewport
    }

    pub(crate) fn setup(&self) -> &Setup {
        &self.setup
    }
}
