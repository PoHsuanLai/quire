//! A menu's highlight goes out when the pointer leaves the menu, as a macOS menu's does: a key
//! still moves it, a menu the pointer never entered keeps the one it opened on, the pointer back
//! in lights the row under it, and the parent row of an open submenu stays lit.

use dioxus::prelude::*;
use ds::host::measure::Anchor;
use ds::prelude::*;
use ds_harness::{Clock, Harness, HarnessConfig, Input, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 640,
    height: 400,
    scale_percent: 100,
};

/// Far from the menu, on the page.
const AWAY: Point = Point {
    x: Px(600.0),
    y: Px(380.0),
};

const ROOT: &str = ".ds-menu:not([*|data-depth])";

fn item(value: u8, title: &str) -> MenuItem<u8> {
    MenuItem::new(value, title)
}

/// Three items, the third a parent of two.
fn entries() -> Vec<MenuItem<u8>> {
    vec![
        item(1, "Open"),
        item(2, "Quit"),
        MenuItem::Submenu {
            title: "More".to_string(),
            image: None,
            availability: Availability::Enabled,
            children: vec![item(10, "About"), item(11, "Help")],
        },
    ]
}

#[component]
fn Page(start: Option<usize>, expanded: Option<usize>) -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "height:380px" }
            Menu::<u8> {
                placement: MenuPlacement::Popup,
                anchor: Anchor::Point(Point { x: Px(40.0), y: Px(60.0) }),
                items: entries(),
                start,
                expanded,
                onpick: |_| {},
                onclose: |_| {},
            }
        }
    }
}

#[allow(non_snake_case)]
fn Plain() -> Element {
    rsx! { Page { start: None, expanded: None } }
}

#[allow(non_snake_case)]
fn OnSecond() -> Element {
    rsx! { Page { start: Some(1), expanded: None } }
}

#[allow(non_snake_case)]
fn Expanded() -> Element {
    rsx! { Page { start: None, expanded: Some(2) } }
}

fn open(app: fn() -> Element) -> Harness {
    let mut harness = Harness::new(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(300));
    harness
}

fn row(harness: &Harness, n: usize) -> Point {
    harness
        .centre(&format!(
            "{ROOT} .ds-menu-item:nth-child({n}) .ds-menu-label"
        ))
        .unwrap_or_else(|| panic!("no row {n}:\n{}", harness.html()))
}

/// The 1-based rows of the root menu drawn highlighted.
fn lit(harness: &Harness) -> Vec<usize> {
    (1..=3)
        .filter(|n| {
            harness
                .attr(
                    &format!("{ROOT} .ds-menu-item:nth-child({n})"),
                    "data-selected",
                )
                .is_some()
        })
        .collect()
}

fn settle(harness: &mut Harness, input: Input) {
    harness.send(input);
    harness.advance(Duration::from_millis(16));
}

#[test]
fn leaving_the_menu_puts_the_highlight_out_and_coming_back_lights_the_row_under_the_pointer() {
    let mut harness = open(Plain);
    assert_eq!(lit(&harness), Vec::<usize>::new(), "it opens on nothing");
    let second = row(&harness, 2);
    settle(&mut harness, Input::pointer_move(second));
    assert_eq!(lit(&harness), [2], "the pointer lights its row");
    settle(&mut harness, Input::pointer_move(AWAY));
    assert_eq!(
        lit(&harness),
        Vec::<usize>::new(),
        "out of the menu, nothing is lit"
    );
    settle(&mut harness, Input::key(ShortcutKey::Down));
    assert_eq!(lit(&harness), [1], "a key starts again from the top");
    let first = row(&harness, 1);
    settle(&mut harness, Input::pointer_move(first));
    settle(&mut harness, Input::pointer_move(second));
    assert_eq!(
        lit(&harness),
        [2],
        "back in, the row under the pointer lights"
    );
}

#[test]
fn a_menu_the_pointer_never_entered_keeps_its_highlight() {
    let mut harness = open(OnSecond);
    assert_eq!(lit(&harness), [2], "it opens on its chosen value");
    settle(&mut harness, Input::pointer_move(AWAY));
    settle(
        &mut harness,
        Input::pointer_move(Point {
            x: Px(590.0),
            y: Px(370.0),
        }),
    );
    assert_eq!(
        lit(&harness),
        [2],
        "a pointer that never came in changes nothing"
    );
    settle(&mut harness, Input::key(ShortcutKey::Down));
    assert_eq!(lit(&harness), [3], "the keyboard moves it");
}

#[test]
fn the_parent_of_an_open_submenu_stays_lit_when_the_pointer_leaves() {
    let mut harness = open(Expanded);
    assert_eq!(
        harness.count(".ds-menu[*|data-depth]"),
        1,
        "the submenu is open"
    );
    let parent = row(&harness, 3);
    settle(&mut harness, Input::pointer_move(parent));
    assert_eq!(lit(&harness), [3]);
    settle(
        &mut harness,
        Input::pointer_move(Point {
            x: Px(10.0),
            y: Px(380.0),
        }),
    );
    assert_eq!(lit(&harness), [3], "it owns the open submenu");
    assert_eq!(
        harness.count(".ds-menu[*|data-depth]"),
        1,
        "which stays open"
    );
}
