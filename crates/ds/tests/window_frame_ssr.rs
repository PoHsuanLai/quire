//! The window frame as markup (FINDINGS "Window frame"): a `Ds` root with
//! `window: WindowFrame::Titlebar` and the lights shown, with them hidden, and in a maximized
//! window (a host that reports `Maximized::On`: the green light restores, the edges are gone),
//! each through dioxus-ssr against a golden under `controls/window_frame/`, so the control goldens'
//! markup lint covers them. A root with no frame is every other root golden, unchanged.
//!
//! `DS_BLESS=1 cargo test -p ds --test window_frame_ssr` rewrites these goldens.

#[path = "support/golden.rs"]
mod golden;

use dioxus::prelude::*;
use ds::{
    Appearance, Ds, HostWindow, Inject, Material, Maximized, ResizeEdge, Support, TileError,
    TrafficLights, WindowFrame, WindowState, WindowTile, Zoom, use_window_host_provider,
};
use std::rc::Rc;

/// A host that does nothing and reports `maximized`.
struct Still(Maximized);

impl HostWindow for Still {
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
        WindowState {
            maximized: self.0,
            ..WindowState::default()
        }
    }
}

fn window(lights: TrafficLights) -> Element {
    rsx! {
        Ds {
            appearance: Appearance::default(),
            material: Material::Window,
            stylesheet: Inject::Host,
            window: WindowFrame::titlebar("Inbox — 3 unread", lights),
            p { "The app" }
        }
    }
}

const CASES: &[(&str, fn() -> Element)] = &[
    ("controls/window_frame/lights.html", || {
        window(TrafficLights::Shown)
    }),
    ("controls/window_frame/no-lights.html", || {
        window(TrafficLights::Hidden)
    }),
    ("controls/window_frame/maximized.html", || {
        use_window_host_provider(|| Rc::new(Still(Maximized::On)));
        window(TrafficLights::Shown)
    }),
];

#[derive(Props, Clone)]
struct HostProps {
    make: fn() -> Element,
}

/// Never equal: the host renders once.
impl PartialEq for HostProps {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

fn host(props: HostProps) -> Element {
    (props.make)()
}

fn render(make: fn() -> Element) -> String {
    let mut dom = VirtualDom::new_with_props(host, HostProps { make });
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn every_frame_state_matches_its_golden() {
    let failures: Vec<String> = CASES
        .iter()
        .filter_map(|(name, make)| golden::check(name, &render(*make)).err())
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn a_maximized_window_has_no_edges_and_its_green_light_restores() {
    let html = render(CASES[2].1);
    assert!(!html.contains("ds-resize-edge"), "{html}");
    assert!(html.contains(r#"aria-label="Restore""#), "{html}");
    let normal = render(CASES[0].1);
    assert_eq!(normal.matches("ds-resize-edge").count(), 8, "{normal}");
    assert!(normal.contains(r#"aria-label="Zoom""#), "{normal}");
}
