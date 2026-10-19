//! A source list's selected row is a neutral grey wash (`--sel-neutral`), not the accent, whether
//! or not the list holds the keyboard, in an active window and an inactive one: a sidebar's
//! selection says where you are, and the accent is left to the content (design/27 section 5.12).

use crate::support::probe;

use dioxus::prelude::*;
use ds::prelude::*;
use ds::style::tokens::accent_table::accent_of;
use ds::style::tokens::hex::Hex;
use ds_harness::{Clock, Harness, HarnessConfig, Viewport};
use probe::{distance, keep, modal, pixels, rect};

const VIEW: Viewport = Viewport {
    width: 300,
    height: 180,
    scale_percent: 100,
};

/// A source list whose one row is selected, its list the keyboard's (no `data-focus`) or not
/// (`away`).
fn list(id: &'static str, focus: Option<&'static str>) -> Element {
    rsx! {
        div { class: "ds-list", "data-style": "source-list", "data-focus": focus,
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
            div { style: "padding:8px", {list("focused", None)} }
            div { style: "padding:8px", {list("active", Some("away"))} }
            div { class: "ds", "data-theme": "light", "data-activity": "inactive", style: "padding:8px", {list("inactive", Some("away"))} }
        }
    }
}

#[test]
fn the_selected_row_is_neutral_grey_focused_or_not_active_or_inactive() {
    let mut harness = Harness::new(Stage, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    let frame = harness.render().expect("renders");
    keep(&frame, "source-list-selection");
    let fill = |id: &str| modal(&pixels(&frame, rect(&harness, id), 2.0));
    let Hex([r, g, b]) = accent_of(Accent::Blue, Scheme::Light).fill;
    for id in ["#focused", "#active", "#inactive"] {
        let got = fill(id);
        assert!(
            got[0].abs_diff(got[2]) < 24,
            "{id} row {got:?} is not a grey"
        );
        assert!(
            distance(got, [r, g, b, 255]) > 60,
            "{id} row {got:?} is the accent"
        );
    }
}
