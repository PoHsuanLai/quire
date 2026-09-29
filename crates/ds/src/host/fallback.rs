//! What the host found for a click.

use dioxus::prelude::MountedData;
use std::rc::Rc;

/// What the host found for a click.
#[derive(Clone)]
pub enum Fallback {
    /// The renderer's default stands: it focuses what was clicked, or it keeps the focus, or
    /// there is no focusable ancestor to move it to.
    Renderer,
    /// The renderer is about to clear the focus; this ancestor should have it once the click is
    /// done.
    Ancestor(Rc<MountedData>),
}

impl std::fmt::Debug for Fallback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Fallback::Renderer => "Renderer",
            Fallback::Ancestor(_) => "Ancestor",
        })
    }
}
