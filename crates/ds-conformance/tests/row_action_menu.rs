//! A row's action with a menu hung from it (design/30 section 2.6), on a real Blitz document: the
//! action hands its button's element to the caller through `common.mounted`, so a menu the caller
//! builds stands under it; an overflow (`PopUpKind::Overflow` in the row's slot) carries the menu
//! itself, opened and closed by the button. Neither picks the row.

use dioxus::prelude::*;
use ds::components::menus::pop_up_button::{PopUpButton, PopUpKind};
use ds::host::measure::{Anchor, MountedRef};
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::ControlSize;
use ds_harness::harness::settle_until;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 640,
    height: 400,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn entries() -> Vec<MenuItem<usize>> {
    vec![
        MenuItem::new(0, "Rename"),
        MenuItem::new(1, "Mark all as read"),
        MenuItem::new(2, "Delete"),
    ]
}

/// What the row kept of its button, for a menu of the caller's own.
#[component]
fn OwnMenu() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut button = use_signal(|| None::<MountedRef>);
    let mut open = use_signal(|| false);
    let action = RowAction::new(
        Icon::Ellipsis,
        "More",
        EventHandler::new(move |_| open.set(true)),
    )
    .with_common(Common {
        mounted: Some(EventHandler::new(move |event: MountedEvent| {
            button.set(Some(MountedRef(event.data())));
        })),
        ..Common::default()
    });
    rsx! {
        p { class: "log", {log().join(",")} }
        div { style: "width:260px;margin:60px 0 0 40px",
            Row {
                title: "Receipts",
                action,
                onclick: move |_| log.with_mut(|log| log.push("row".to_string())),
            }
        }
        if let (true, Some(anchor)) = (open(), button()) {
            Menu::<usize> {
                placement: MenuPlacement::Popup,
                anchor: Anchor::Mounted(anchor),
                items: entries(),
                onpick: move |value: usize| log.with_mut(|log| log.push(format!("own:{value}"))),
                onclose: move |()| open.set(false),
            }
        }
    }
}

/// An overflow: the row's own menu, a pop-up button of its own kind in the row's slot.
#[component]
fn Overflow() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    rsx! {
        p { class: "log", {log().join(",")} }
        div { style: "width:260px;margin:60px 0 0 40px",
            Row {
                title: "Receipts",
                accessory: Accessory::Slot(rsx! {
                    PopUpButton::<usize> {
                        kind: PopUpKind::Overflow,
                        size: ControlSize::Small,
                        items: entries(),
                        onpick: move |value: usize| log.with_mut(|log| log.push(format!("more:{value}"))),
                    }
                }),
                onclick: move |_| log.with_mut(|log| log.push("row".to_string())),
            }
        }
    }
}

#[allow(non_snake_case)]
fn OwnPage() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            OwnMenu {}
        }
    }
}

#[allow(non_snake_case)]
fn OverflowPage() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            Overflow {}
        }
    }
}

fn started(app: fn() -> Element) -> Harness {
    let mut harness = Harness::new(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(200));
    harness
}

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

const OWN: &str = ".ds-row-action .ds-button";
const MORE: &str = ".ds-row-trailing .ds-popup .ds-button";

#[test]
fn a_menu_the_caller_builds_hangs_from_the_actions_button() {
    let mut harness = started(OwnPage);
    let at = harness.centre(OWN).expect("the action");
    harness.send(Input::click(at));
    harness.advance(ms(200));
    assert_eq!(harness.count(".ds-menu"), 1, "the caller's menu is up");
    let button = harness.rect(OWN).expect("the button");
    let menu = harness.rect(".ds-menu").expect("the menu");
    assert!(
        menu.origin.y.0 >= button.origin.y.0 + button.size.height.0 - 12.0
            && menu.origin.y.0 <= button.origin.y.0 + button.size.height.0 + 12.0,
        "it stands under the button: {menu:?} {button:?}"
    );
    assert_eq!(log(&harness), "", "the row was not picked");
}

#[test]
fn an_overflow_opens_its_menu_under_the_button_and_a_pick_reaches_on_pick() {
    let mut harness = started(OverflowPage);
    assert_eq!(
        harness.attr(MORE, "aria-expanded").as_deref(),
        Some("false")
    );
    let at = harness.centre(MORE).expect("the overflow");
    harness.send(Input::click(at));
    harness.advance(ms(200));
    assert_eq!(harness.count(".ds-menu"), 1);
    assert_eq!(harness.attr(MORE, "aria-expanded").as_deref(), Some("true"));
    let button = harness.rect(MORE).expect("the button");
    let menu = harness.rect(".ds-menu").expect("the menu");
    assert!(
        (menu.origin.y.0 - (button.origin.y.0 + button.size.height.0)).abs() < 12.0,
        "under the button: {menu:?} {button:?}"
    );
    let second = harness
        .centre(".ds-menu-item:nth-child(2) .ds-menu-label")
        .expect("the second item");
    harness.send(Input::pointer_move(second));
    harness.send(Input::click(second));
    settle_until(&mut harness, |h| h.count(".ds-menu") == 0);
    assert_eq!(
        log(&harness),
        "more:1",
        "the pick, and the row never picked"
    );
    assert_eq!(
        harness.attr(MORE, "aria-expanded").as_deref(),
        Some("false")
    );
}

#[test]
fn pressing_the_overflow_again_closes_its_menu_without_a_pick() {
    let mut harness = started(OverflowPage);
    let at = harness.centre(MORE).expect("the overflow");
    harness.send(Input::click(at));
    harness.advance(ms(200));
    assert_eq!(harness.count(".ds-menu"), 1);
    // The menu's layer catches the press outside it; the button is outside the menu.
    harness.send(Input::click(at));
    settle_until(&mut harness, |h| h.count(".ds-menu") == 0);
    assert_eq!(log(&harness), "");
}
