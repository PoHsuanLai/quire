//! A menu item that keeps its menu open, and the trailing hint, on a real Blitz document
//! (design/30 section 2.4): an `AfterPick::KeepOpen` item yields its value at once, with no blink
//! and no close, as often as it is picked, by the pointer or Return; an ordinary item still
//! blinks, yields and fades the menu out. A hint draws at the item's end, before its key
//! equivalent.

use dioxus::prelude::*;
use ds::host::measure::Anchor;
use ds::prelude::*;
use ds_harness::harness::settle_until;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 720,
    height: 480,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// Two labels that toggle in place, and a Done that closes the menu.
fn items(on: &[usize]) -> Vec<MenuItem<usize>> {
    let label = |value: usize, name: &str| {
        let state = if on.contains(&value) {
            Check::On
        } else {
            Check::Off
        };
        MenuItem::new(value, name)
            .with_check(state)
            .with_after(AfterPick::KeepOpen)
    };
    vec![
        label(0, "Invoices"),
        label(1, "Travel"),
        MenuItem::Separator,
        MenuItem::new(2, "Done")
            .with_hint("closes")
            .with_key(Shortcut(vec![ShortcutKey::Super, ShortcutKey::Char('d')])),
    ]
}

#[allow(non_snake_case)]
fn Page() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut on = use_signal(Vec::<usize>::new);
    let mut open = use_signal(|| true);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "height:460px; padding:20px",
                p { class: "log", {log().join(",")} }
            }
            if open() {
                Menu::<usize> {
                    placement: MenuPlacement::Popup,
                    anchor: Anchor::Point(Point { x: Px(40.0), y: Px(60.0) }),
                    items: items(&on()),
                    onpick: move |value: usize| {
                        log.with_mut(|log| log.push(format!("pick:{value}")));
                        if value < 2 {
                            on.with_mut(|on| match on.iter().position(|at| *at == value) {
                                Some(at) => {
                                    on.remove(at);
                                }
                                None => on.push(value),
                            });
                        }
                    },
                    onclose: move |()| {
                        open.set(false);
                        log.with_mut(|log| log.push("close".to_string()));
                    },
                }
            }
        }
    }
}

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

fn row(harness: &Harness, n: usize) -> Point {
    harness
        .centre(&format!(".ds-menu-item:nth-child({n}) .ds-menu-label"))
        .unwrap_or_else(|| panic!("no row {n}:\n{}", harness.html()))
}

fn checked(harness: &Harness, n: usize) -> Option<String> {
    harness.attr(&format!(".ds-menu-item:nth-child({n})"), "aria-checked")
}

fn started() -> Harness {
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(300));
    harness
}

#[test]
fn a_keep_open_item_yields_at_once_and_leaves_the_menu_up() {
    let mut harness = started();
    assert_eq!(checked(&harness, 1).as_deref(), Some("false"));
    let first = row(&harness, 1);
    harness.send(Input::pointer_move(first));
    harness.send(Input::click(first));
    // No blink to wait out: the value is there on the next frame, and so is the new state.
    harness.advance(ms(20));
    assert_eq!(log(&harness), "pick:0");
    assert_eq!(
        checked(&harness, 1).as_deref(),
        Some("true"),
        "the caller's toggle shows"
    );
    assert_eq!(harness.count(".ds-menu"), 1, "the menu is still up");
    assert!(
        harness
            .attr(".ds-menu-item:nth-child(1)", "data-selected")
            .is_some(),
        "the highlight never went out: nothing blinked"
    );
    // Again: picking a toggle twice flips it back, which a closed menu could not allow.
    harness.send(Input::click(first));
    harness.advance(ms(20));
    assert_eq!(log(&harness), "pick:0,pick:0");
    assert_eq!(checked(&harness, 1).as_deref(), Some("false"));
    assert_eq!(harness.count(".ds-menu"), 1);
}

#[test]
fn return_on_a_keep_open_item_picks_it_and_keeps_the_menu() {
    let mut harness = started();
    let second = row(&harness, 2);
    harness.send(Input::pointer_move(second));
    harness.send(Input::key(ShortcutKey::Enter));
    harness.advance(ms(20));
    assert_eq!(log(&harness), "pick:1");
    assert_eq!(harness.count(".ds-menu"), 1);
}

#[test]
fn an_ordinary_item_still_blinks_yields_and_closes_after_toggles() {
    let mut harness = started();
    let first = row(&harness, 1);
    harness.send(Input::pointer_move(first));
    harness.send(Input::click(first));
    harness.advance(ms(20));
    let done = row(&harness, 4);
    harness.send(Input::pointer_move(done));
    harness.send(Input::click(done));
    harness.advance(ms(20));
    assert_eq!(log(&harness), "pick:0", "Done waits out its blink");
    settle_until(&mut harness, |h| log(h).ends_with("close"));
    assert_eq!(log(&harness), "pick:0,pick:2,close");
    assert_eq!(harness.count(".ds-menu"), 0);
}

#[test]
fn a_hint_draws_before_the_key_equivalent_at_the_items_end() {
    let harness = started();
    let hint = harness
        .centre(".ds-menu-item:nth-child(4) .ds-menu-hint")
        .expect("the hint");
    let keys = harness
        .centre(".ds-menu-item:nth-child(4) .ds-menu-trailing")
        .expect("the key equivalent");
    let title = harness
        .centre(".ds-menu-item:nth-child(4) .ds-menu-label")
        .expect("the title");
    assert_eq!(harness.text_of(".ds-menu-hint").as_deref(), Some("closes"));
    assert!(
        title.x < hint.x && hint.x < keys.x,
        "title, hint, keys in order"
    );
}
