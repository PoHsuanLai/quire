//! A source list's selected row is the accent under white ink while its window is active, even
//! when the list does not hold the keyboard, and the quiet grey while the window is inactive
//! (macOS's sidebar).

#[path = "support/probe.rs"]
mod probe;

use dioxus::prelude::*;
use ds::prelude::*;
use ds::style::tokens::accent_table::accent_of;
use ds::style::tokens::hex::Hex;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Viewport};
use probe::{distance, keep, modal, pixels, rect};

const VIEW: Viewport = Viewport {
    width: 300,
    height: 140,
    scale_percent: 100,
};

/// A source list whose one row is selected and whose list is not the keyboard's.
fn list(id: &'static str) -> Element {
    rsx! {
        div { class: "ds-list", "data-style": "source-list", "data-focus": "away",
            div { id, class: "ds-row", "data-focus": "highlight", "aria-selected": "true", style: "height:28px", "Inbox" }
        }
    }
}

#[allow(non_snake_case)]
fn Stage() -> Element {
    rsx! {
        Ds {
            appearance: Appearance { theme: Theme::Light, ..Appearance::default() },
            material: Material::Window,
            div { style: "padding:8px", {list("active")} }
            div { class: "ds", "data-theme": "light", "data-activity": "inactive", style: "padding:8px", {list("inactive")} }
        }
    }
}

#[test]
fn the_selected_row_is_accent_while_active_and_grey_while_inactive() {
    let mut harness = Harness::new(Stage, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    let frame = harness.render().expect("renders");
    keep(&frame, "source-list-selection");
    let fill = |id: &str| modal(&pixels(&frame, rect(&harness, id), 2.0));
    let blue = accent_of(Accent::Blue, Scheme::Light).fill;
    let Hex([r, g, b]) = blue;
    assert!(
        distance(fill("#active"), [r, g, b, 255]) <= 2,
        "active row {:?}, want the accent {blue:?}",
        fill("#active")
    );
    let quiet = fill("#inactive");
    assert!(
        quiet[0].abs_diff(quiet[2]) < 24,
        "inactive row {quiet:?} is not a grey"
    );
}
