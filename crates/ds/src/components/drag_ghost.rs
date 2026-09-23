//! DragGhost, DropLine and Grip: moving a thing by dragging, driven by `use_drag`
//! (design/04-COMPONENTS.md section 34).

use crate::geometry::{Point, Rect};
use crate::geometry::measure::client_rect;
use dioxus::prelude::*;
use std::rc::Rc;

/// Where the ghost's corner sits relative to the pointer: `(x - 40, y - 18)` (`C:2053-2054`).
const GHOST_OFFSET: Point = Point {
    x: crate::geometry::Px(-40.0),
    y: crate::geometry::Px(-18.0),
};

/// The ghost's `left` and `top` for a pointer at `at`.
fn ghost_style(at: Point) -> String {
    let left = at.x.0 + GHOST_OFFSET.x.0;
    let top = at.y.0 + GHOST_OFFSET.y.0;
    format!("left:{left}px;top:{top}px")
}

/// The tilted card that follows the pointer. `at` is the pointer, as a live
/// `DragPhase::Live { at, .. }` reports it; the card sits 40 px left of it and 18 px above.
/// Render it through the `OverlayHost` (it is `position:fixed`).
#[component]
pub fn DragGhost(title: String, sub: String, at: Point) -> Element {
    rsx! {
        div { class: "ds-drag-ghost", style: ghost_style(at),
            "{title}"
            div { class: "ds-drag-ghost-sub ds-truncate", "{sub}" }
        }
    }
}

/// Where a dragged object will land.
#[component]
pub fn DropLine() -> Element {
    rsx! {
        div { class: "ds-drop-line" }
    }
}

/// An object's drag handle; clicking it opens the object menu at its rect.
///
/// The rect is read in the click handler, after layout, never in `onmounted` (spike S9).
#[component]
pub fn Grip(label: String, onclick: EventHandler<Rect>) -> Element {
    let mut element = use_signal(|| None::<Rc<MountedData>>);
    rsx! {
        span {
            class: "ds-grip",
            role: "button",
            tabindex: "0",
            title: "{label}",
            "aria-label": "{label}",
            onmounted: move |event| element.set(Some(event.data())),
            onclick: move |_| {
                if let Some(mounted) = element() {
                    spawn(async move {
                        if let Some(measured) = client_rect(&mounted).await {
                            onclick.call(measured);
                        }
                    });
                }
            },
            "⋮⋮"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ghost_style;
    use crate::geometry::{Point, Px};

    #[test]
    fn the_ghost_sits_up_and_left_of_the_pointer() {
        const CASES: &[(f32, f32, &str)] = &[
            (452.0, 206.0, "left:412px;top:188px"),
            (40.0, 18.0, "left:0px;top:0px"),
            (10.5, 0.0, "left:-29.5px;top:-18px"),
        ];
        for &(x, y, want) in CASES {
            let at = Point { x: Px(x), y: Px(y) };
            assert_eq!(ghost_style(at), want, "pointer at {x},{y}");
        }
    }
}
