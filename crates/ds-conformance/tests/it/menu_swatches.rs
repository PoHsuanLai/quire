//! A row of swatches in a menu, on a real Blitz document: each dot is its own target named for its
//! tag; Up and Down take the row as one stop, Left and Right walk its dots; a click picks the dot
//! under the pointer.

use dioxus::prelude::*;
use ds::host::measure::Anchor;
use ds::prelude::*;
use ds::style::tokens::system_hue::SystemHue;
use ds_harness::harness::settle_until;
use ds_harness::{Clock, Harness, HarnessConfig, Input, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 720,
    height: 480,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// A command, a row of three tag dots (Red is on), and another command.
#[allow(non_snake_case)]
fn Page() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let items = vec![
        MenuItem::new(0u8, "New Folder"),
        MenuItem::swatches(
            "Tags",
            vec![
                MenuSwatch::new(1, "Red", SystemHue::Red).with_check(Check::On),
                MenuSwatch::new(2, "Blue", SystemHue::Blue),
                MenuSwatch::new(3, "Green", SystemHue::Green),
            ],
        ),
        MenuItem::new(9u8, "Delete"),
    ];
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            p { class: "log", {log().join(",")} }
            Menu::<u8> {
                placement: MenuPlacement::Context,
                anchor: Anchor::Point(Point { x: Px(40.0), y: Px(60.0) }),
                items,
                onpick: move |value: u8| log.with_mut(|log| log.push(format!("pick:{value}"))),
                onclose: move |()| log.with_mut(|log| log.push("close".to_string())),
            }
        }
    }
}

fn opened() -> Harness {
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(300));
    harness
}

/// The name of the dot the highlight is on.
fn dot(harness: &Harness) -> Option<String> {
    harness.attr(".ds-menu-dot[*|data-selected=true]", "aria-label")
}

/// The row the highlight is on, when it is an item.
fn row(harness: &Harness) -> Option<String> {
    harness.text_of(".ds-menu-item[*|data-selected=true] .ds-menu-label")
}

#[test]
fn each_dot_is_named_for_its_tag_and_the_row_is_a_group() {
    let harness = opened();
    assert_eq!(harness.count(".ds-menu-dot"), 3);
    assert_eq!(
        harness.attr(".ds-menu-swatches", "aria-label").as_deref(),
        Some("Tags")
    );
    assert_eq!(
        harness
            .attr(".ds-menu-dot:nth-child(2)", "aria-label")
            .as_deref(),
        Some("Blue")
    );
    assert_eq!(
        harness
            .attr(".ds-menu-dot:nth-child(1)", "aria-checked")
            .as_deref(),
        Some("true")
    );
}

#[test]
fn up_and_down_pass_the_row_as_one_stop_and_left_and_right_walk_the_dots() {
    let mut harness = opened();
    harness.send(Input::key(ShortcutKey::Down));
    assert_eq!(row(&harness).as_deref(), Some("New Folder"));
    harness.send(Input::key(ShortcutKey::Down));
    assert_eq!(dot(&harness).as_deref(), Some("Red"));
    harness.send(Input::key(ShortcutKey::Right));
    assert_eq!(dot(&harness).as_deref(), Some("Blue"));
    harness.send(Input::key(ShortcutKey::Right));
    harness.send(Input::key(ShortcutKey::Right));
    assert_eq!(dot(&harness).as_deref(), Some("Green"), "stops at the last");
    harness.send(Input::key(ShortcutKey::Down));
    assert_eq!(row(&harness).as_deref(), Some("Delete"));
    assert_eq!(dot(&harness), None);
    harness.send(Input::key(ShortcutKey::Up));
    assert_eq!(dot(&harness).as_deref(), Some("Red"), "back on the first");
}

#[test]
fn a_click_picks_the_dot_under_the_pointer() {
    let mut harness = opened();
    let green = harness
        .centre(".ds-menu-dot:nth-child(3)")
        .expect("the third dot");
    harness.send(Input::pointer_move(green));
    assert_eq!(dot(&harness).as_deref(), Some("Green"));
    harness.send(Input::click(green));
    settle_until(&mut harness, |h| {
        h.text_of(".log").is_some_and(|log| log.ends_with("close"))
    });
    assert_eq!(harness.text_of(".log").as_deref(), Some("pick:3,close"));
}
