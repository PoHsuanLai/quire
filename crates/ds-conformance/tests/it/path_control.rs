//! A path bar (design/30 section 4): a click on a segment reports its index, a long path keeps
//! its root and last folders and lists the middle ones behind a button, and a plain click on a
//! toolbar title with a title menu opens nothing.

use dioxus::prelude::*;
use ds::components::chrome::toolbar::view::Toolbar;
use ds::prelude::*;
use ds_harness::{Clock, Harness, HarnessConfig, Input, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 700,
    height: 300,
    scale_percent: 100,
};

fn folders(names: &[&str]) -> Vec<PathSegment> {
    names
        .iter()
        .map(|name| PathSegment::new(*name, Icon::Folder))
        .collect()
}

#[allow(non_snake_case)]
fn Page() -> Element {
    let mut said = use_signal(|| "none".to_owned());
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            PathControl {
                label: "Path",
                segments: folders(&["Root", "Users", "me", "Documents", "Projects", "Quire", "Notes"]),
                onpick: move |at: usize| said.set(format!("picked {at}")),
            }
            Toolbar::<u8> {
                title: Some(TextLine::from("Notes")),
                title_menu: folders(&["Quire", "Projects"]),
                onpick: |_| {},
            }
            p { id: "said", "{said}" }
        }
    }
}

fn start() -> Harness {
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(200));
    harness
}

fn click(harness: &mut Harness, selector: &str) {
    let at = harness.centre(selector).expect(selector);
    harness.send(Input::click(at));
    harness.advance(Duration::from_millis(400));
}

#[test]
fn a_long_path_collapses_its_middle_and_keeps_both_ends() {
    let harness = start();
    let html = harness.html();
    assert!(
        harness.centre(".ds-path-more").is_some(),
        "one collapse button"
    );
    assert!(html.contains("Root") && html.contains("Notes"));
    assert!(!html.contains("Users"), "the middle is behind the button");
}

#[test]
fn a_click_on_a_segment_reports_its_index_in_the_whole_path() {
    let mut harness = start();
    click(&mut harness, ".ds-path-segment:last-of-type");
    assert!(harness.html().contains("picked 6"));
}

#[test]
fn a_hidden_segment_is_picked_from_the_collapse_button() {
    let mut harness = start();
    click(&mut harness, ".ds-path-more");
    assert!(
        harness.centre(".ds-menu").is_some(),
        "the menu lists the middle"
    );
    click(&mut harness, ".ds-menu-item:nth-of-type(2)");
    assert!(harness.html().contains("picked 2"));
}

#[test]
fn a_plain_click_on_a_title_with_a_menu_opens_none() {
    let mut harness = start();
    click(&mut harness, ".ds-toolbar-title");
    assert!(harness.centre(".ds-menu").is_none(), "no menu opened");
}
