//! Where a mouse or pointer event happened, in the client coordinates every quire rect is read in.

use dioxus::prelude::*;
use ds_core::geometry::units::{Point, Px};

/// Where a mouse event happened, in the client coordinates every quire rect is read in.
pub(crate) fn client_point(event: &MouseEvent) -> Point {
    let at = event.client_coordinates();
    Point {
        x: Px(at.x as f32),
        y: Px(at.y as f32),
    }
}

/// Where a pointer event happened, in the same client coordinates.
pub(crate) fn pointer_point(event: &PointerEvent) -> Point {
    let at = event.client_coordinates();
    Point {
        x: Px(at.x as f32),
        y: Px(at.y as f32),
    }
}
