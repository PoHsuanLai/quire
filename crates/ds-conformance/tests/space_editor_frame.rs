//! SpaceEditor without its own card (`frame: EditorFrame::Frameless`): a host that is a surface
//! already (a Sheet) draws it with no border, ground, shadow or padding, so its first row starts
//! at the editor's own edge; the default is still the card, whose rows sit inside its padding.

#[path = "support/probe.rs"]
mod probe;

use dioxus::prelude::*;
use ds::components::app::space_editor::DotIndex;
use ds::components::app::space_editor::rows::EditorFrame;
use ds::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Viewport};
use probe::{keep, rect};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 760,
    height: 700,
    scale_percent: 100,
};

/// The card editor and the frameless one side by side, each in its own host, in `theme`.
fn page(theme: Theme, scheme: Scheme) -> Element {
    let look = SpaceLook::default();
    rsx! {
        Ds { appearance: Appearance { theme, ..Appearance::default() }, material: Material::Window,
            div { style: "display:flex; gap:24px; padding:16px; align-items:flex-start",
                div { class: "card", style: "width:340px",
                    SpaceEditor { look: look.clone(), scheme, active_dot: DotIndex(0), onchange: |_| {} }
                }
                div { class: "bare", style: "width:340px; padding:16px; background:var(--raise); border-radius:var(--r-menu)",
                    SpaceEditor { look, scheme, active_dot: DotIndex(0), onchange: |_| {}, frame: EditorFrame::Frameless }
                }
            }
        }
    }
}

#[allow(non_snake_case)]
fn Light() -> Element {
    page(Theme::Light, Scheme::Light)
}

#[allow(non_snake_case)]
fn Dark() -> Element {
    page(Theme::Dark, Scheme::Dark)
}

/// How far the title starts inside the editor's box, on each axis.
fn inset(harness: &Harness, scope: &str) -> (f32, f32) {
    let editor = rect(harness, &format!("{scope} .ds-space-editor"));
    let title = rect(harness, &format!("{scope} .ds-space-editor-title"));
    (
        title.origin.x.0 - editor.origin.x.0,
        title.origin.y.0 - editor.origin.y.0,
    )
}

#[test]
fn a_frameless_editor_has_no_card_around_its_rows() {
    for (page, shot) in [(Light as fn() -> Element, "light"), (Dark, "dark")] {
        check(page, shot);
    }
}

fn check(page: fn() -> Element, shot: &str) {
    let mut harness = Harness::new(page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(50));
    let frame = harness.render().expect("the page renders");
    keep(&frame, &format!("space-editor-frame-{shot}"));
    let (card_x, card_y) = inset(&harness, ".card");
    assert!(
        card_x >= 14.0 && card_y >= 14.0,
        "the card pads its rows: {card_x}, {card_y}"
    );
    assert_eq!(
        inset(&harness, ".bare"),
        (0.0, 0.0),
        "the frameless one does not"
    );
    let card_w = rect(&harness, ".card .ds-space-editor").size.width.0;
    let bare_w = rect(&harness, ".bare .ds-space-editor").size.width.0;
    assert!(
        (card_w - 340.0).abs() < 1.0 && (bare_w - 308.0).abs() < 1.0,
        "the frameless editor fills its host's content box: {card_w} and {bare_w}"
    );
}
