//! Input that reaches a menu row after its owner unmounted must not panic. A floating menu is drawn
//! by the overlay host, but its handlers belong to the `Menu` scope; when a pick closes the menu
//! the scope's handlers are dropped a frame before the host takes the rows out, and a mouse move,
//! release or click in that frame once called a dropped `EventHandler`
//! (`Dropped(ValueDroppedError)`, mailo's "Always allow" pop-up on a Settings form row).
//!
//! The harness settles every render pass before the next event, so it cannot hold the window's
//! one-frame gap open; these tests drive the sequences around it (a pick that closes, an unmount
//! with the pointer on a row) at scale 2 and pin that no event after the close panics. The guard
//! itself is unit-tested in `ds::components::menus::alive`.

use dioxus::prelude::*;
use ds::components::menus::pop_up_button::{PopUpButton, PopUpKind};
use ds::host::measure::Anchor;
use ds::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::cell::RefCell;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 480,
    height: 320,
    scale_percent: 200,
};

thread_local! {
    static OPEN: RefCell<Option<Signal<bool>>> = const { RefCell::new(None) };
}

fn items() -> Vec<MenuItem<u8>> {
    (0..3u8)
        .map(|value| MenuItem::Item {
            value,
            title: format!("Command {value}"),
            image: None,
            key: None,
            check: None,
            availability: Availability::Enabled,
            hint: None,
            after: AfterPick::Close,
            text: Default::default(),
        })
        .collect()
}

#[allow(non_snake_case)]
fn Unmounting() -> Element {
    let open = use_signal(|| true);
    OPEN.with(|slot| *slot.borrow_mut() = Some(open));
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "height:100vh" }
            if open() {
                Menu {
                    placement: MenuPlacement::Context,
                    anchor: Anchor::Point(Point { x: Px(100.0), y: Px(60.0) }),
                    items: items(),
                    onpick: move |_: u8| {},
                    onclose: move |_| {},
                }
            }
        }
    }
}

#[allow(non_snake_case)]
fn FormRow() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "padding:24px",
                PopUpButton::<u8> {
                    items: items(),
                    onpick: |_| {},
                    kind: PopUpKind::PopUp,
                    value: Some(0u8),
                    start: Shown::Visible,
                    anchor: Some(Anchor::Point(Point { x: Px(120.0), y: Px(80.0) })),
                }
            }
        }
    }
}

fn started(app: fn() -> Element) -> Harness {
    let mut harness = Harness::new(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    for _ in 0..10 {
        harness.advance(Duration::from_millis(100));
    }
    harness
}

fn moved(at: Point) -> Point {
    Point {
        x: Px(at.x.0 + 1.0),
        y: at.y,
    }
}

#[test]
fn input_over_a_row_as_its_menu_unmounts_does_not_panic() {
    let mut harness = started(Unmounting);
    let row = harness.centre(".ds-menu-item").expect("a row is drawn");
    harness.send(Input::pointer_move(row));
    harness.within(|| OPEN.with(|slot| slot.borrow().expect("app built").set(false)));
    harness.key_pending(ShortcutKey::Shift);
    let _ = harness.step_frame();
    harness.send(Input::pointer_move(moved(row)));
    harness.send(Input::click(moved(row)));
    harness.advance(Duration::from_millis(100));
    assert_eq!(harness.count(".ds-menu-item"), 0, "{}", harness.html());
}

#[test]
fn input_after_a_pick_closes_a_pop_up_on_a_form_row_does_not_panic() {
    let mut harness = started(FormRow);
    let row = harness.centre(".ds-menu-item").expect("a row is drawn");
    harness.send(Input::pointer_move(row));
    harness.send(Input::click(row));
    // The pick blinks, fades and closes; input keeps arriving over the place the rows were.
    for _ in 0..12 {
        harness.send(Input::pointer_move(moved(row)));
        harness.send(Input::click(moved(row)));
        harness.advance(Duration::from_millis(30));
    }
    harness.advance(Duration::from_millis(500));
    assert_eq!(harness.count(".ds-menu-item"), 0, "{}", harness.html());
}
