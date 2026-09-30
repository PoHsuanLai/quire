//! DragImage: the ghost of one item and of several, pinned near the top right while shown.

use crate::axes::{Axes, Showcase};
use crate::pages::Section;
use dioxus::prelude::*;
use ds::components::overlays::drag_ghost::DragCount;
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;
use ds_core::press::Press;

/// The DragGhost section.
#[component]
pub fn DragGhostSection() -> Element {
    let showcase = use_context::<Signal<Axes>>()().showcase;
    let mut ghost = use_signal(|| match showcase {
        Showcase::Posed => Check::On,
        Showcase::Live => Check::Off,
    });
    rsx! {
        Section { title: "DragImage", note: "NSDraggingItem: the ghost follows the pointer 1:1, flat, 40 px left of it and 18 above; a drag of several things carries a count badge on its corner. The ghost is fixed to the window: here one and five are pinned near the top right.",
            div { class: "g-row",
                Button {
                    size: ControlSize::Mini,
                    label: "Show the ghosts",
                    value: Some(ghost()),
                    onclick: move |_: Press| ghost.set(ghost().flipped()),
                }
            }
            if ghost() == Check::On {
                DragGhost { title: "Quarterly report.pdf", sub: "2.4 MB", at: Point { x: Px(760.0), y: Px(120.0) } }
                DragGhost { title: "5 items", sub: "Downloads", count: DragCount::new(5), at: Point { x: Px(1040.0), y: Px(120.0) } }
            }
        }
    }
}
