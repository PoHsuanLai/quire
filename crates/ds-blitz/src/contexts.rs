//! The app's own root contexts, handed to the window, the harness and a snapshot alike, so a
//! component reads the same `use_context` value in all three (mailo's store, its appearance, the
//! `mailo open` target), with no global to smuggle them past `launch`'s `fn() -> Element`.

use dioxus::prelude::VirtualDom;
use std::any::Any;
use std::fmt;
use std::sync::Arc;

/// Makes one fresh copy of a context each time a document is built.
type Factory = Arc<dyn Fn() -> Box<dyn Any> + Send + Sync>;

/// Values provided at the root of the app's tree, each read with `use_context::<T>()`.
///
/// `Send + Sync` because dioxus-native builds the window's tree on the event-loop thread from
/// factories handed over at `launch`; `Clone` because `use_context` hands out clones.
#[derive(Clone, Default)]
pub struct RootContexts {
    factories: Vec<Factory>,
}

impl RootContexts {
    /// No contexts.
    pub fn new() -> Self {
        RootContexts::default()
    }

    /// These contexts and `value`, provided as `T` at the root. A later value of the same type
    /// shadows an earlier one, as nested `use_context_provider`s do.
    pub fn with<T: Clone + Send + Sync + 'static>(mut self, value: T) -> Self {
        self.factories
            .push(Arc::new(move || Box::new(value.clone()) as Box<dyn Any>));
        self
    }

    /// These contexts, then every one of `more`.
    pub fn and(mut self, more: RootContexts) -> Self {
        self.factories.extend(more.factories);
        self
    }

    /// Provide every value at `vdom`'s root, in the order they were added.
    pub(crate) fn install(&self, vdom: &mut VirtualDom) {
        for factory in &self.factories {
            vdom.insert_any_root_context(factory());
        }
    }
}

impl fmt::Debug for RootContexts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RootContexts")
            .field("len", &self.factories.len())
            .finish()
    }
}
