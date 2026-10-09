//! The edges a host reports as tiled reach a component through `use_window_tiled`, and a host
//! that says nothing reports none.

use dioxus::prelude::*;
use ds::prelude::*;
use ds::window::host::{use_window_host_provider, use_window_tiled};
use ds::window::tiled::{TileEdge, Tiled};
use ds::window::vocab::{ResizeEdge, Support, TileError, WindowTile, Zoom};
use ds_core::word::Word;
use std::rc::Rc;

/// A host that does nothing and reports `tiled`.
struct Tiling(Tiled);

impl HostWindow for Tiling {
    fn begin_move(&self) {}
    fn begin_resize(&self, _: ResizeEdge) {}
    fn zoom(&self, _: Zoom) {}
    fn minimize(&self) {}
    fn close(&self) {}
    fn tile(&self, _: WindowTile) -> Result<(), TileError> {
        Err(TileError::Unsupported)
    }
    fn supports(&self, _: WindowTile) -> Support {
        Support::No
    }
    fn state(&self) -> WindowState {
        WindowState::default()
    }
    fn tiled(&self) -> Tiled {
        self.0
    }
}

fn reads() -> Element {
    let words: Vec<&str> = use_window_tiled().edges().map(|e| e.slug()).collect();
    rsx! { p { "{words.join(\",\")}" } }
}

fn left_and_top() -> Element {
    use_window_host_provider(|| {
        Rc::new(Tiling(Tiled::NONE.with(TileEdge::Left).with(TileEdge::Top)))
    });
    reads()
}

fn silent() -> Element {
    reads()
}

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn a_host_reports_its_tiled_edges_and_no_host_reports_none() {
    assert_eq!(render(left_and_top), "<p>top,left</p>");
    assert_eq!(render(silent), "<p></p>");
}
