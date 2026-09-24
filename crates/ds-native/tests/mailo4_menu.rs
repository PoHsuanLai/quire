//! mailo gaps 4, menus, on a real Blitz document: a toggle checklist whose picks keep it open
//! (`dismiss: PickDismiss::Stay`) toggles each row's check with the cursor staying on it, and
//! still closes on Escape and an outside press; a menu drawn inline in a card sits in the
//! card, off the overlay and the layer stack, and picks as a menu does.

use dioxus::prelude::*;
use ds::components::vocab::Check;
use ds::{
    Anchor, Appearance, Ds, Flow, Key, Material, Menu, MenuEntry, MenuKind, MenuRow, PickDismiss,
    Point, Px,
};
use ds_native::{Harness, Viewport};
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

/// The label rows, checked as `on` says.
fn rows(on: &[Check; 3]) -> Vec<MenuEntry<usize>> {
    LABELS
        .into_iter()
        .enumerate()
        .map(|(value, name)| {
            MenuEntry::Row(MenuRow {
                check: Some(on[value]),
                ..MenuRow::new(value, name)
            })
        })
        .collect()
}

fn flip(check: Check) -> Check {
    match check {
        Check::Checked => Check::Unchecked,
        Check::Unchecked => Check::Checked,
    }
}

/// A label picker, floating or drawn in its card, with its picks' dismissal.
#[component]
fn Page(dismiss: PickDismiss, flow: Flow) -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut on = use_signal(|| [Check::Checked, Check::Unchecked, Check::Unchecked]);
    let mut open = use_signal(|| true);
    let menu = rsx! {
        Menu::<usize> {
            kind: MenuKind::Dropdown,
            anchor: Anchor::Point(Point { x: Px(40.0), y: Px(60.0) }),
            entries: rows(&on()),
            onpick: move |value: usize| {
                on.with_mut(|on| on[value] = flip(on[value]));
                log.with_mut(|log| log.push(format!("pick:{value}")));
            },
            onclose: move |()| {
                open.set(false);
                log.with_mut(|log| log.push("close".to_string()));
            },
            dismiss,
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
fn Staying() -> Element {
    rsx! { Page { dismiss: PickDismiss::Stay, flow: Flow::Floating } }
}

#[allow(non_snake_case)]
fn Closing() -> Element {
    rsx! { Page { dismiss: PickDismiss::Close, flow: Flow::Floating } }
}

#[allow(non_snake_case)]
fn Inline() -> Element {
    rsx! { Page { dismiss: PickDismiss::Close, flow: Flow::Inline } }
}

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

/// The `aria-checked` of row `n` (1-based).
fn checked(harness: &Harness, n: usize) -> Option<String> {
    harness.attr(&format!(".ds-menu-item:nth-child({n})"), "aria-checked")
}

fn highlighted(harness: &Harness) -> Option<String> {
    harness.text_of(".ds-menu-item[*|aria-selected=true] .ds-menu-title")
}

fn row(harness: &Harness, n: usize) -> Point {
    harness
        .centre(&format!(".ds-menu-item:nth-child({n}) .ds-menu-title"))
        .unwrap_or_else(|| panic!("no row {n}:\n{}", harness.html()))
}

#[test]
fn a_stay_pick_toggles_the_check_and_keeps_the_menu_and_its_cursor() {
    let mut harness = Harness::new(Staying, VIEW);
    harness.advance(ms(300));
    let second = row(&harness, 2);
    harness.pointer_move(second);
    harness.click(second);
    harness.advance(ms(300));
    assert_eq!(log(&harness), "pick:1");
    assert_eq!(harness.count(".ds-menu"), 1, "still open");
    assert_eq!(checked(&harness, 2).as_deref(), Some("true"));
    assert_eq!(highlighted(&harness).as_deref(), Some("Travel"));
    // Again: it toggles back, and the menu stays.
    harness.click(second);
    harness.advance(ms(100));
    assert_eq!(log(&harness), "pick:1,pick:1");
    assert_eq!(checked(&harness, 2).as_deref(), Some("false"));
    // Blitz clears the focus at the end of a click; the menu takes it back, for the keys.
    assert!(harness.is_focused(".ds-menu"), "the menu kept the keyboard");
    // Enter picks the row under the cursor, which stayed on Travel.
    harness.key(Key::Enter);
    harness.advance(ms(100));
    assert_eq!(log(&harness), "pick:1,pick:1,pick:1");
    assert_eq!(checked(&harness, 2).as_deref(), Some("true"));
    assert_eq!(harness.count(".ds-menu"), 1);
    // Escape still closes, after its fade.
    harness.key(Key::Escape);
    harness.advance(ms(300));
    assert_eq!(log(&harness), "pick:1,pick:1,pick:1,close");
    assert_eq!(harness.count(".ds-menu"), 0);
}

#[test]
fn an_outside_press_still_closes_a_stay_menu() {
    let mut harness = Harness::new(Staying, VIEW);
    harness.advance(ms(300));
    harness.click(row(&harness, 3));
    harness.advance(ms(50));
    harness.click(Point {
        x: Px(650.0),
        y: Px(420.0),
    });
    harness.advance(ms(300));
    assert_eq!(log(&harness), "pick:2,close");
    assert_eq!(harness.count(".ds-menu"), 0);
}

#[test]
fn a_close_pick_still_closes() {
    let mut harness = Harness::new(Closing, VIEW);
    harness.advance(ms(300));
    harness.click(row(&harness, 2));
    harness.advance(ms(300));
    assert_eq!(log(&harness), "pick:1,close");
    assert_eq!(harness.count(".ds-menu"), 0);
}

#[test]
fn an_inline_menu_stands_in_its_card_off_the_overlay() {
    let mut harness = Harness::new(Inline, VIEW);
    harness.advance(ms(300));
    assert_eq!(harness.count(".card > .ds-menu[*|data-flow=inline]"), 1);
    assert_eq!(harness.count(".ds-overlay .ds-menu"), 0);
    assert_eq!(harness.count(".ds-overlay-catch"), 0, "no outside catcher");
    assert!(
        !harness.is_focused(".ds-menu"),
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
    harness.key(Key::Escape);
    harness.advance(ms(300));
    assert_eq!(log(&harness), "");
    harness.click(row(&harness, 3));
    harness.advance(ms(100));
    assert_eq!(log(&harness), "pick:2,close");
}
