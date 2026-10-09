//! The host's rect read (`ds::host::parts::GeometryHost::measure`): a mounted element's border box straight from the
//! Blitz document, answering "busy" instead of panicking when the renderer holds the document.
//!
//! dioxus-native-dom's own `get_client_rect` borrows the document mutably. A task that dioxus
//! polls inside `render_immediate` (it does, when the task woke in the same turn as a dirty
//! scope) runs while the mutation writer holds that borrow, and the read panics with "RefCell
//! already borrowed" (the hover card's anchor read, 450 ms after the pointer
//! came to rest). Reading through `NodeHandle::try_doc` turns that into `Measured::Busy`, and
//! quire's reader waits a frame.
//!
//! Every Blitz host has it: `launch` and the harness wire it, and a shell's surface root gets it
//! from [`provide_host`](crate::provide_host).

use crate::node_ref::NodeRef;
use crate::phase::border_box;
use dioxus::prelude::*;
use ds::prelude::*;

/// `element`'s border box, if the document is free and the element is a Blitz node: a component's
/// own mounted handle, or an element found by selector (`GeometryHost::find`).
pub fn measure(element: &MountedData) -> Measured {
    let Some(node) = NodeRef::of(element) else {
        return Measured::Unknown;
    };
    match node.read(|doc| border_box(doc, node.node)) {
        None => Measured::Busy,
        Some(Some(rect)) => Measured::At(rect),
        Some(None) => Measured::Unknown,
    }
}
