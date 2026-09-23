//! DragGhost, DropLine and Grip: moving a thing by dragging, driven by `use_drag`
//! (design/04-COMPONENTS.md section 34).

use crate::geometry::{Point, Rect};
use dioxus::prelude::*;

/// The tilted card that follows the pointer.
#[component]
pub fn DragGhost(title: String, sub: String, at: Point) -> Element {
    todo!()
}

/// Where a dragged object will land.
#[component]
pub fn DropLine() -> Element {
    todo!()
}

/// An object's drag handle; clicking it opens the object menu at its rect.
#[component]
pub fn Grip(label: String, onclick: EventHandler<Rect>) -> Element {
    todo!()
}
