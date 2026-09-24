//! What a document is given beyond quire's own contexts, shared by the window (`AppConfig`)
//! and the headless document (`HarnessConfig`), so a window, a test and a snapshot of the same
//! app see the same providers.

use crate::contexts::RootContexts;

/// The app's providers for one document.
#[derive(Debug, Clone, Default)]
pub(crate) struct Setup {
    /// Values provided at the root, read with `use_context`.
    pub(crate) contexts: RootContexts,
}
