//! The host's rect read (`ds::HostMeasure`): a mounted element's border box straight from the
//! Blitz document, answering "busy" instead of panicking when the renderer holds the document.
//!
//! dioxus-native-dom's own `get_client_rect` borrows the document mutably. A task that dioxus
//! polls inside `render_immediate` (it does, when the task woke in the same turn as a dirty
//! scope) runs while the mutation writer holds that borrow, and the read panics with "RefCell
//! already borrowed" (wave 2 integration: the hover card's anchor read, 450 ms after the pointer
//! came to rest). Reading through `NodeHandle::try_doc` turns that into `Measured::Busy`, and
//! quire's reader waits a frame.
//!
//! Any host that runs a Blitz document provides it: `launch` and the harness do, and a host
//! that is not `ds_native::launch` (shell-host's surfaces, sill's bar and its popups) calls
//! [`provide`] at the top of its root component (sill FINDINGS Q10).

use dioxus::prelude::*;
use dioxus_native_dom::NodeHandle;
use ds::{HostMeasure, Measured, Point, Px, Rect, Size};

/// The Blitz rect read, as the `ds::HostMeasure` a root provides as context. `launch` and the
/// harness provide it already; another host provides it with [`provide`] or
/// `use_context_provider(|| ds_native::measure::MEASURE)`.
pub const MEASURE: HostMeasure = HostMeasure(measure);

/// Provide [`MEASURE`] to the calling component's subtree. Call it at the top of a root that
/// `ds_native::launch` did not start (a shell-host surface's component, a popup's document),
/// before any quire component reads a rect.
pub fn provide() -> HostMeasure {
    use_context_provider(|| MEASURE)
}

/// `element`'s border box, if the document is free and the element is a Blitz node.
fn measure(element: &MountedData) -> Measured {
    let Some(handle) = element.downcast::<NodeHandle>() else {
        return Measured::Unknown;
    };
    let Some(doc) = handle.try_doc() else {
        return Measured::Busy;
    };
    match doc.get_client_bounding_rect(handle.node_id()) {
        Some(found) => Measured::At(Rect {
            origin: Point {
                x: Px(found.x as f32),
                y: Px(found.y as f32),
            },
            size: Size {
                width: Px(found.width as f32),
                height: Px(found.height as f32),
            },
        }),
        None => Measured::Unknown,
    }
}
