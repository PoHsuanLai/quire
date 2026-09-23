//! The host's rect read (`ds::HostMeasure`): a mounted element's border box straight from the
//! Blitz document, answering "busy" instead of panicking when the renderer holds the document.
//!
//! dioxus-native-dom's own `get_client_rect` borrows the document mutably. A task that dioxus
//! polls inside `render_immediate` (it does, when the task woke in the same turn as a dirty
//! scope) runs while the mutation writer holds that borrow, and the read panics with "RefCell
//! already borrowed" (wave 2 integration: the hover card's anchor read, 450 ms after the pointer
//! came to rest). Reading through `NodeHandle::try_doc` turns that into `Measured::Busy`, and
//! quire's reader waits a frame.

use dioxus::prelude::*;
use dioxus_native_dom::NodeHandle;
use ds::{HostMeasure, Measured, Point, Px, Rect, Size};

/// The measurer `Host` and `Headless` provide as root context.
pub(crate) const MEASURE: HostMeasure = HostMeasure(measure);

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
