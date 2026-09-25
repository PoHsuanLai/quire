//! The Polish page's window frame (FINDINGS "Window frame"): the titlebar with its traffic
//! lights in an active window with the tiling menu posed open (Left half, Right half and Centre
//! unavailable, as on Wayland, where a client cannot place itself), in an inactive window (grey
//! lights), and maximized. Each specimen's host is a stub that does nothing and reports its state.

use super::Specimen;
use super::polish::Root;
use crate::pages::Section;
use dioxus::prelude::*;
use ds::{
    Activation, HostWindow, Material, Maximized, ResizeEdge, Support, TileError, TilePose,
    WindowState, WindowTile, WindowTitlebar, Zoom, use_window_host_provider,
};
use std::rc::Rc;

/// A host that reports `state` and can only fill, as a Wayland window's.
struct Posed(WindowState);

impl HostWindow for Posed {
    fn begin_move(&self) {}
    fn begin_resize(&self, _: ResizeEdge) {}
    fn zoom(&self, _: Zoom) {}
    fn minimize(&self) {}
    fn close(&self) {}
    fn tile(&self, _: WindowTile) -> Result<(), TileError> {
        Err(TileError::Unsupported)
    }
    fn supports(&self, tile: WindowTile) -> Support {
        match tile {
            WindowTile::Fill => Support::Yes,
            WindowTile::LeftHalf | WindowTile::RightHalf | WindowTile::Centre => Support::No,
        }
    }
    fn state(&self) -> WindowState {
        self.0
    }
}

/// One framed window: a Window root with the titlebar over a line of body text.
#[component]
fn Framed(title: String, state: WindowState, pose: TilePose, height: u32) -> Element {
    use_window_host_provider(move || Rc::new(Posed(state)));
    rsx! {
        Root { material: Material::Window, style: "width:460px",
            // The root is the menu's bounds: as tall as the window, so the menu hangs under
            // the light instead of being pushed up over it.
            div { style: "height:{height}px",
                WindowTitlebar { title, pose }
                p { class: "g-note", style: "padding:0 var(--s-13)", "The window's content." }
            }
        }
    }
}

/// The section.
#[component]
pub fn WindowFrameSection() -> Element {
    let active = WindowState::default();
    let inactive = WindowState {
        activated: Activation::Inactive,
        ..active
    };
    let maximized = WindowState {
        maximized: Maximized::On,
        ..active
    };
    rsx! {
        Section { title: "Window frame", note: "WindowTitlebar, what Ds {{ window: WindowFrame::Titlebar }} draws: a 28 px titlebar on the frame, the title 13/600 centred, the lights 12 px, 8 apart, 13 from the edge. Coloured in the active window, grey in an inactive one; the marks show while the pointer is over the group. The green light's tiling menu is posed open: Fill, and the three placements a Wayland client cannot make, unavailable.",
            div { class: "g-row g-row-top",
                Specimen { name: "Active, tiling menu open",
                    code: "macOS: lights 12 pt, 8 apart; Sequoia's Move & Resize menu on the green light".to_owned(),
                    Framed { title: "Inbox", state: active, pose: TilePose::Open, height: 200 }
                }
                Specimen { name: "Inactive",
                    code: "grey until the pointer is over the group".to_owned(),
                    Framed { title: "Drafts", state: inactive, pose: TilePose::Closed, height: 90 }
                }
                Specimen { name: "Maximized",
                    code: "no resize edges, no drag; the green light restores".to_owned(),
                    Framed { title: "Archive", state: maximized, pose: TilePose::Closed, height: 90 }
                }
            }
        }
    }
}
