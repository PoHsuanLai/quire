//! What a document is given beyond quire's own contexts, shared by the window (`AppConfig`)
//! and the headless document (`HarnessConfig`), so a window, a test and a snapshot of the same
//! app see the same providers.

use crate::click_focus::FocusFallback;
use crate::contexts::RootContexts;
use crate::frame_links::FrameLinks;
use crate::net_policy::NetPolicy;
use dioxus::prelude::VirtualDom;
use ds::keys::KeySource;

/// The app's providers for one document.
#[derive(Debug, Clone, Default)]
pub struct Setup {
    /// Values provided at the root, read with `use_context`.
    pub contexts: RootContexts,
    /// Who answers the document's requests, and its frames'.
    pub net: NetPolicy,
    /// What a link clicked in a frame does.
    pub frame_links: FrameLinks,
    /// Where the keyboard goes after a click on nothing focusable.
    pub focus_fallback: FocusFallback,
    /// Where the keymap comes from, provided at the root for `Ds` to load. Our desktop's
    /// conventions until the launch names another.
    pub keys: KeySource,
}

impl Setup {
    /// Provide the keymap source and then the app's contexts at `vdom`'s root. The app's own
    /// values come last, so a `KeySource` the app provides itself wins.
    pub fn install(&self, vdom: &mut VirtualDom) {
        vdom.insert_any_root_context(Box::new(self.keys.clone()));
        self.contexts.install(vdom);
    }
}
