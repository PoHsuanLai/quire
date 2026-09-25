//! The frame's resize edges: a thin grab zone along each side and a larger one at each corner,
//! over the window's own edge (a client-decorated window has no border outside its content to
//! put them in). A primary press on one asks the host to resize from that edge at once, since the
//! compositor takes the pointer from the press. Not drawn while the window is maximized or
//! fullscreen, where resizing is the compositor's.

use crate::window::{ResizeEdge, use_window_host, use_window_state};
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;

/// The eight grab zones, or nothing while the window fills its output.
#[component]
pub(crate) fn ResizeEdges() -> Element {
    let host = use_window_host();
    if !use_window_state().movable() {
        return rsx! {};
    }
    rsx! {
        for edge in ResizeEdge::ALL {
            div {
                key: "{edge.slug()}",
                class: "ds-resize-edge",
                "data-edge": edge.slug(),
                "data-first-mouse": "",
                onpointerdown: {
                    let host = host.clone();
                    move |event: PointerEvent| {
                        if event.trigger_button() == Some(MouseButton::Primary)
                            && let Some(host) = host.as_ref()
                        {
                            event.stop_propagation();
                            host.host().begin_resize(edge);
                        }
                    }
                },
            }
        }
    }
}
