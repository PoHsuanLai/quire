//! Menus, on a real Blitz document: a pick blinks its item twice, then yields its value and fades
//! the menu out; a menu drawn inline in a card sits in the card, off the overlay and the layer
//! stack, and picks as a menu does.

use dioxus::prelude::*;
use ds::host::measure::Anchor;
use ds::prelude::*;
use ds::style::tokens::delay::DelayToken;
use ds_harness::harness::settle_until;
use ds_harness::{Clock, Driver, FocusState, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 720,
    height: 480,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

const LABELS: [&str; 3] = ["Invoices", "Travel", "Family"];

/// The label items.
fn items() -> Vec<MenuItem<usize>> {
    LABELS
        .into_iter()
        .enumerate()
        .map(|(value, name)| MenuItem::new(value, name))
        .collect()
}

/// A label picker, floating or drawn in its card.
#[component]
fn Page(flow: Flow) -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut open = use_signal(|| true);
    let menu = rsx! {
        Menu::<usize> {
            placement: MenuPlacement::Popup,
            anchor: Anchor::Point(Point { x: Px(40.0), y: Px(60.0) }),
            items: items(),
            onpick: move |value: usize| log.with_mut(|log| log.push(format!("pick:{value}"))),
            onclose: move |()| {
                open.set(false);
                log.with_mut(|log| log.push("close".to_string()));
            },
            flow,
        }
    };
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "height:460px; padding:20px",
                p { class: "log", {log().join(",")} }
                div { class: "card", style: "width:240px; margin-top:200px",
                    if open() && flow == Flow::Inline {
                        {menu.clone()}
                    }
                }
            }
            if open() && flow == Flow::Floating {
                {menu}
            }
        }
    }
}

#[allow(non_snake_case)]
fn Floating() -> Element {
    rsx! { Page { flow: Flow::Floating } }
}

#[allow(non_snake_case)]
fn Inline() -> Element {
    rsx! { Page { flow: Flow::Inline } }
}

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

fn row(harness: &Harness, n: usize) -> Point {
    harness
        .centre(&format!(".ds-menu-item:nth-child({n}) .ds-menu-label"))
        .unwrap_or_else(|| panic!("no row {n}:\n{}", harness.html()))
}

/// Whether row `n` (1-based) is drawn highlighted.
fn lit(harness: &Harness, n: usize) -> bool {
    harness
        .attr(&format!(".ds-menu-item:nth-child({n})"), "data-selected")
        .is_some()
}

#[test]
fn a_pick_blinks_its_item_twice_then_yields_and_closes() {
    let mut harness = Harness::new(
        Floating,
        HarnessConfig::new(VIEW).with_clock(Clock::Virtual),
    );
    harness.advance(ms(300));
    let second = row(&harness, 2);
    harness.send(Input::pointer_move(second));
    assert!(lit(&harness, 2), "the pointer highlights the row");
    harness.send(Input::click(second));
    // The blink is out, back, out, back: each half is half of `MenuBlink` (35 ms).
    let half = DelayToken::MenuBlink.delay() / 2;
    let mut seen = Vec::new();
    for _ in 0..4 {
        seen.push(lit(&harness, 2));
        assert_eq!(log(&harness), "", "nothing is yielded during the blink");
        harness.advance(half);
    }
    assert_eq!(seen, [false, true, false, true], "out, back, out, back");
    let closed = settle_until(&mut harness, |h| log(h).ends_with("close"));
    assert_eq!(log(&harness), "pick:1,close");
    assert!(closed > harness.now() - ms(1000));
    assert_eq!(harness.count(".ds-menu"), 0);
}

#[test]
fn a_second_pick_during_the_blink_is_ignored() {
    let mut harness = Harness::new(
        Floating,
        HarnessConfig::new(VIEW).with_clock(Clock::Virtual),
    );
    harness.advance(ms(300));
    harness.send(Input::click(row(&harness, 2)));
    harness.send(Input::click(row(&harness, 3)));
    settle_until(&mut harness, |h| log(h).ends_with("close"));
    assert_eq!(log(&harness), "pick:1,close");
}

#[test]
fn an_outside_press_closes_without_a_pick() {
    let mut harness = Harness::new(
        Floating,
        HarnessConfig::new(VIEW).with_clock(Clock::Virtual),
    );
    harness.advance(ms(300));
    harness.send(Input::click(Point {
        x: Px(650.0),
        y: Px(420.0),
    }));
    harness.advance(ms(300));
    assert_eq!(log(&harness), "close");
    assert_eq!(harness.count(".ds-menu"), 0);
}

#[test]
fn an_inline_menu_stands_in_its_card_off_the_overlay() {
    let mut harness = Harness::new(Inline, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(300));
    assert_eq!(harness.count(".card > .ds-menu[*|data-flow=inline]"), 1);
    assert_eq!(harness.count(".ds-overlay .ds-menu"), 0);
    assert_eq!(harness.count(".ds-overlay-catch"), 0, "no outside catcher");
    assert_eq!(
        harness.focus_of(".ds-menu"),
        FocusState::Unfocused,
        "the caller keeps the focus"
    );
    // The rows sit inside the card, where the flow put them.
    let card = harness.rect(".card").expect("the card");
    let first = harness.rect(".ds-menu-item").expect("a row");
    assert!(
        first.origin.y.0 >= card.origin.y.0 && first.origin.x.0 >= card.origin.x.0,
        "{first:?} in {card:?}"
    );
    // Escape is the caller's: nothing closes.
    harness.send(Input::key(ShortcutKey::Escape));
    harness.advance(ms(300));
    assert_eq!(log(&harness), "");
    harness.send(Input::click(row(&harness, 3)));
    settle_until(&mut harness, |h| log(h).ends_with("close"));
    assert_eq!(log(&harness), "pick:2,close");
}
