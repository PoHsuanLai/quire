//! The layer under a floating surface that catches the press outside it. It covers the whole
//! overlay, or everything but one element: a field the surface hangs from keeps its own presses,
//! so a press on it neither closes the surface nor takes the keyboard from it.

use dioxus::prelude::*;
use ds_core::geometry::units::Rect;

/// What the outside-press catcher leaves alone.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Catcher {
    /// Nothing: every press outside the surface is caught.
    Whole,
    /// Nothing yet: the element to leave alone is not laid out, and covering it would take the
    /// press that opened the surface (a press that opens a panel mid-press lands on whatever is
    /// over the field when it is released).
    Pending,
    /// This rect, in the overlay's own coordinates: a press on it reaches the element under it.
    Around(Rect),
}

impl Catcher {
    /// The catcher's elements: one covering the overlay, or four around the hole.
    pub(crate) fn draw(self, onpress: EventHandler<()>) -> Element {
        match self {
            Catcher::Pending => rsx! {},
            Catcher::Whole => rsx! {
                div { class: "ds-overlay-catch", onpointerdown: move |_| onpress.call(()) }
            },
            Catcher::Around(hole) => {
                let (x, y) = (hole.origin.x.0, hole.origin.y.0);
                let (right, below) = (x + hole.size.width.0, y + hole.size.height.0);
                let height = hole.size.height.0;
                // Above and below span the overlay's width; left and right fill the hole's rows.
                let parts = [
                    format!("left:0;right:0;top:0;height:{y}px"),
                    format!("left:0;right:0;top:{below}px;bottom:0"),
                    format!("left:0;width:{x}px;top:{y}px;height:{height}px"),
                    format!("left:{right}px;right:0;top:{y}px;height:{height}px"),
                ];
                rsx! {
                    for (at , style) in parts.into_iter().enumerate() {
                        div {
                            key: "{at}",
                            class: "ds-overlay-catch-part",
                            style,
                            onpointerdown: move |_| onpress.call(()),
                        }
                    }
                }
            }
        }
    }
}
