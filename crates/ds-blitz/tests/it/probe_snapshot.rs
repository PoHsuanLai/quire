//! The debug probe's lines: a control by role and accessible name, with its box in the window's
//! logical pixels, and what it says about its state.

use dioxus::prelude::*;
use ds_blitz::seam::probe_snapshot;
use ds_harness::{Clock, DocQuery, Harness, HarnessConfig, Viewport};

const VIEW: Viewport = Viewport {
    width: 400,
    height: 200,
    scale_percent: 100,
};

#[allow(non_snake_case)]
fn Page() -> Element {
    rsx! {
        div { style: "padding:20px",
            button { style: "box-sizing:border-box;width:80px;height:30px", "Manage" }
            button { style: "width:40px;height:30px", "aria-label": "Close", span { "x" } }
            div { role: "switch", "aria-checked": "true", "aria-label": "Mail", style: "width:50px;height:20px" }
            div { "aria-hidden": "true", button { "Hidden" } }
            div { role: "none", style: "width:10px;height:10px" }
        }
    }
}

fn lines() -> Vec<Vec<String>> {
    let harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness
        .with_doc(probe_snapshot)
        .lines()
        .map(|line| line.split('\t').map(str::to_owned).collect())
        .collect()
}

#[test]
fn controls_are_listed_by_role_and_name_with_their_boxes() {
    let found = lines();
    let manage = found
        .iter()
        .find(|line| line[0] == "button" && line[6] == "Manage");
    let manage = manage.unwrap_or_else(|| panic!("no Manage button in {found:?}"));
    assert_eq!(manage[3], "80.0");
    assert_eq!(manage[4], "30.0");
    assert_eq!(manage[5], "-");
    assert!(
        found
            .iter()
            .any(|line| line[0] == "button" && line[6] == "Close")
    );
}

#[test]
fn an_explicit_role_and_its_state_are_kept_and_hidden_or_role_none_are_not() {
    let found = lines();
    let switch = found
        .iter()
        .find(|line| line[0] == "switch")
        .expect("a switch");
    assert_eq!(switch[5], "checked");
    assert_eq!(switch[6], "Mail");
    assert!(found.iter().all(|line| line[6] != "Hidden"), "{found:?}");
    assert!(found.iter().all(|line| line[0] != "none"), "{found:?}");
}
