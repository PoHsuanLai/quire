//! The dock's own pieces (the macOS polish pass, 2026-09-24; design/10-BEHAVIOUR-dock.md
//! section 10.3.2): the running dot under a tile and the optional reflective floor inside the
//! pill. The shell lays out the tiles; these read the dock tokens (`--dock-dot`,
//! `--dock-dot-gap`, `--dock-floor`) its `DockMetrics` writes.

use dioxus::prelude::*;

/// The running dot: a `--dock-dot` (4 px) circle centred under its tile, `--dock-dot-gap`
/// (3 px) below the tile's bottom edge, in the dock's ink; it fades in over `--t-quick`. Place
/// it inside the tile's box (a positioned element), and leave it out when the app has no
/// window.
#[component]
pub fn RunningDot() -> Element {
    rsx! {
        span { class: "ds-running-dot", "aria-hidden": "true" }
    }
}

/// The reflective floor: a soft light rising from the pill's floor, off unless `dock.floor` is
/// on (`--dock-floor`). Place it first inside the dock's root, under the tiles.
#[component]
pub fn DockFloor() -> Element {
    rsx! {
        div { class: "ds-dock-floor", "aria-hidden": "true" }
    }
}
