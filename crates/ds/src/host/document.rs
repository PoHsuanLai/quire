//! The one seam between the design system and the renderer it is drawn in.

use crate::host::no_host::NoHost;
use crate::host::parts::{
    CaretHost, ClickFocusHost, EditHost, FileDropHost, FocusHost, GeometryHost,
};
use dioxus::prelude::try_consume_context;
use std::rc::Rc;

/// What a component asks of the document it is drawn in, split into parts of a few methods each.
/// A host provides one `Rc<dyn DocumentHost>` as root context, every part installed at once, so a
/// root can never hold a subset. The parts a host may not have are optional.
pub trait DocumentHost {
    /// Keyboard focus.
    fn focus(&self) -> &dyn FocusHost;
    /// A field's caret and selection.
    fn caret(&self) -> &dyn CaretHost;
    /// Rects, scrolling and lookup by selector.
    fn geometry(&self) -> &dyn GeometryHost;
    /// The keyboard's fallback after a click on nothing focusable, where the host has one.
    fn click_focus(&self) -> Option<&dyn ClickFocusHost>;
    /// Geometry, the clipboard, the pointer and the IME for an edit surface, where the host has
    /// them.
    fn edit(&self) -> Option<&dyn EditHost>;
    /// The hit test for a file drag, where the host has one.
    fn file_drop(&self) -> Option<&dyn FileDropHost>;
}

/// The enclosing document's host: [`NoHost`] where none was provided (a server render), whose
/// every method answers `Unknown` or is absent.
pub fn use_document_host() -> Rc<dyn DocumentHost> {
    try_consume_context::<Rc<dyn DocumentHost>>().unwrap_or_else(|| Rc::new(NoHost::default()))
}
