//! DragGhost and DropLine: moving a thing by dragging, driven by `use_drag`
//! (design/04-COMPONENTS.md section 34; design/30 section 2.5, `NSDraggingItem`).

use crate::components::controls::badge::{Badge, BadgeContent};
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::geometry::units::Point;
use ds_style::tokens::control_size::ControlSize;

/// Where the ghost's corner sits relative to the pointer: `(x - 40, y - 18)` (`C:2053-2054`).
const GHOST_OFFSET: Point = Point {
    x: ds_core::geometry::units::Px(-40.0),
    y: ds_core::geometry::units::Px(-18.0),
};

/// The ghost's `left` and `top` for a pointer at `at`.
fn ghost_style(at: Point) -> String {
    let left = at.x.0 + GHOST_OFFSET.x.0;
    let top = at.y.0 + GHOST_OFFSET.y.0;
    format!("left:{left}px;top:{top}px")
}

/// How many things a drag carries: one draws the plain ghost, more draw a count badge on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DragCount(u32);

impl DragCount {
    /// A drag of `items` things; none is one.
    pub fn new(items: u32) -> Self {
        DragCount(items.max(1))
    }

    /// The badge's number: only a drag of several shows one.
    fn badge(self) -> Option<u32> {
        (self.0 > 1).then_some(self.0)
    }
}

impl Default for DragCount {
    fn default() -> Self {
        DragCount(1)
    }
}

/// The card that follows the pointer 1:1: no tilt, no lag. `at` is the pointer, as a live
/// `DragPhase::Live { at, .. }` reports it; the card sits 40 px left of it and 18 px above. A drag
/// of several things (`count`) carries a count badge on the card's corner. Render it through the
/// `OverlayHost` (it is `position:fixed`).
#[component]
pub fn DragGhost(
    title: String,
    sub: String,
    at: Point,
    #[props(default)] count: DragCount,
    #[props(default)] common: Common,
) -> Element {
    let data = common.data_attributes();
    let badge = count.badge();
    rsx! {
        div {
            id: common.id.clone(),
            class: common.class("ds-drag-ghost"),
            style: ghost_style(at),
            "data-count": if badge.is_some() { Some("many") } else { None },
            "aria-label": common.aria_label.clone(),
            onmounted: move |event| common.mounted(event),
            ..data,
            span { class: "ds-drag-ghost-title ds-truncate", "{title}" }
            div { class: "ds-drag-ghost-sub ds-truncate", "{sub}" }
            if let Some(number) = badge {
                span { class: "ds-drag-ghost-count",
                    Badge { content: BadgeContent::Number(number), size: ControlSize::Small }
                }
            }
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

#[cfg(test)]
mod tests {
    use super::{DragCount, ghost_style};
    use ds_core::geometry::units::{Point, Px};

    #[test]
    fn only_a_drag_of_several_things_shows_a_badge() {
        const CASES: &[(u32, Option<u32>)] = &[(0, None), (1, None), (2, Some(2)), (14, Some(14))];
        for &(items, want) in CASES {
            assert_eq!(DragCount::new(items).badge(), want, "{items} items");
        }
    }

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
