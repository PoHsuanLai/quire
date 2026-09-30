//! The toast on a real Blitz document (design/30 section 2.9): the pointer over it pauses the
//! hold and leaving starts it over in full; a swipe to the right past 80 px dismisses it, and it
//! leaves by sliding out (`panel-out`) from where the hand let go; the Undo action reports the
//! undo token and dismisses it.

use dioxus::prelude::*;
use ds::{Appearance, Button, Ds, Material, Point, Px, RootExtent, UndoToken, use_toasts};
use ds_native::harness::settle_until;
use ds_native::{Clock, Harness, HarnessConfig, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 800,
    height: 500,
    scale_percent: 100,
};

static UNDONE: GlobalSignal<Vec<u64>> = Signal::global(Vec::new);

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

#[allow(non_snake_case)]
fn Page() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            Pusher {}
        }
    }
}

#[component]
fn Pusher() -> Element {
    let toasts = use_toasts();
    rsx! {
        Button {

            label: "Archive",
            onclick: move |_| toasts.push_undoable(
                "Archived".into(),
                UndoToken(7),
                EventHandler::new(|token: UndoToken| UNDONE.write().push(token.0)),
            ),
        }
    }
}

fn pushed() -> Harness {
    let mut harness =
        Harness::with_config(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.within(|| UNDONE.write().clear());
    let button = harness.centre(".ds-button").expect("the button");
    harness.click(button);
    harness.advance(ms(400));
    assert_eq!(
        harness.attr(".ds-toast", "data-presence").as_deref(),
        Some("present")
    );
    harness
}

fn presence(harness: &Harness) -> Option<String> {
    harness.attr(".ds-toast", "data-presence")
}

#[test]
fn the_pointer_over_it_pauses_the_hold_and_leaving_starts_it_over() {
    let mut harness = pushed();
    let over = harness.centre(".ds-toast").expect("the toast");
    harness.pointer_move(over);
    // Well past the 5 s hold: still up, because the pointer is on it.
    harness.advance(ms(7000));
    assert_eq!(
        presence(&harness).as_deref(),
        Some("present"),
        "paused while it is read"
    );
    harness.pointer_move(Point {
        x: Px(40.0),
        y: Px(40.0),
    });
    harness.advance(ms(2500));
    assert_eq!(
        presence(&harness).as_deref(),
        Some("present"),
        "the hold starts over in full"
    );
    let left = harness.now();
    settle_until(&mut harness, |h| presence(h).as_deref() == Some("leaving"));
    assert!(harness.now().duration_since(left) >= ms(2000));
    harness.advance(ms(400));
    assert_eq!(harness.count(".ds-toast"), 0);
}

#[test]
fn a_swipe_to_the_right_past_80_px_dismisses_it() {
    let mut harness = pushed();
    let from = harness.centre(".ds-toast").expect("the toast");
    // Under the threshold: it springs back and stays.
    harness.drag(
        from,
        Point {
            x: from.x + Px(40.0),
            y: from.y,
        },
        8,
    );
    harness.advance(ms(600));
    assert_eq!(
        presence(&harness).as_deref(),
        Some("present"),
        "under 80 px it springs home"
    );
    let from = harness.centre(".ds-toast").expect("the toast");
    harness.drag(
        from,
        Point {
            x: from.x + Px(140.0),
            y: from.y,
        },
        8,
    );
    harness.advance(ms(20));
    assert_eq!(
        presence(&harness).as_deref(),
        Some("leaving"),
        "past 80 px it goes"
    );
    harness.advance(ms(400));
    assert_eq!(harness.count(".ds-toast"), 0);
}

#[test]
fn undo_reports_its_token_and_dismisses() {
    let mut harness = pushed();
    let action = harness.centre(".ds-toast-action").expect("the action");
    harness.click(action);
    harness.advance(ms(20));
    assert_eq!(harness.within(|| UNDONE.peek().clone()), vec![7]);
    assert_eq!(presence(&harness).as_deref(), Some("leaving"));
}
