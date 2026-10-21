//! A toast pushed from a sheet's own button keeps clear of that button (design/30 section 2.9):
//! while a sheet is up the toast sits at the top right, so a second press on the button reaches
//! the button and not the toast (whose hold pauses under a still pointer). With no sheet it stays
//! at the bottom right. A toast that appears under a still pointer is transparent to it until the
//! pointer moves: a press there reaches the button below and the hold runs; after a move onto it
//! the toast holds and its Undo takes the press.

use dioxus::prelude::*;
use ds::prelude::*;
use ds::root::common::Common;
use ds::stack::toast_hub::UndoToken;
use ds_harness::{Clock, Harness, HarnessConfig, Input, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 900,
    height: 640,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// A sheet whose bottom-right button pushes a toast and counts its presses.
#[allow(non_snake_case)]
fn WithSheet() -> Element {
    let toasts = use_toasts();
    let mut presses = use_signal(|| 0u32);
    rsx! {
        Sheet { label: "Add studio", onclose: |_| {},
            div { style: "height:320px" }
            div { style: "display:flex; justify-content:flex-end; padding:20px",
                Button {
                    label: "Add",
                    common: Common { id: Some("add".to_string()), ..Common::default() },
                    onclick: move |_| {
                        presses += 1;
                        toasts.push(format!("Added {}", presses()), None);
                    },
                }
            }
        }
        p { class: "presses", "{presses}" }
    }
}

/// The same button with no sheet around it.
#[allow(non_snake_case)]
fn NoSheet() -> Element {
    let toasts = use_toasts();
    rsx! {
        Button {
            label: "Add",
            common: Common { id: Some("add".to_string()), ..Common::default() },
            onclick: move |_| toasts.push("Added".into(), None),
        }
    }
}

/// A button at the bottom right, where the toast lands, that pushes an undoable toast and counts
/// its presses and the undos.
#[allow(non_snake_case)]
fn UnderToast() -> Element {
    let toasts = use_toasts();
    let mut presses = use_signal(|| 0u32);
    let mut undos = use_signal(|| 0u32);
    rsx! {
        div { style: "position:fixed; right:20px; bottom:20px; width:140px",
            Button {
                label: "Add computer",
                common: Common { id: Some("add".to_string()), ..Common::default() },
                onclick: move |_| {
                    presses += 1;
                    toasts
                        .push_undoable(
                            "Added".into(),
                            UndoToken(1),
                            EventHandler::new(move |_| undos += 1),
                        );
                },
            }
        }
        p { class: "presses", "{presses}" }
        p { class: "undos", "{undos}" }
    }
}

#[allow(non_snake_case)]
fn UnderPage() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            UnderToast {}
        }
    }
}

#[allow(non_snake_case)]
fn SheetPage() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            WithSheet {}
        }
    }
}

#[allow(non_snake_case)]
fn PlainPage() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            NoSheet {}
        }
    }
}

fn start(page: fn() -> Element) -> Harness {
    let mut harness = Harness::new(page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(600));
    harness
}

#[test]
fn a_toast_over_a_sheet_sits_at_the_top_and_the_button_takes_the_second_press() {
    let mut harness = start(SheetPage);
    let button = harness.rect("#add").expect("the sheet's button");
    let at = harness.centre("#add").expect("its centre");
    harness.send(Input::click(at));
    harness.advance(ms(400));
    assert_eq!(
        harness.attr(".ds-toast", "data-place").as_deref(),
        Some("top")
    );
    let toast = harness.rect(".ds-toast").expect("the toast");
    assert!(
        toast.origin.y.0 + toast.size.height.0 < button.origin.y.0,
        "the toast {toast:?} is clear of the button {button:?}"
    );
    harness.send(Input::click(at));
    harness.advance(ms(100));
    assert_eq!(harness.text_of(".presses").as_deref(), Some("2"));
}

#[test]
fn with_no_sheet_the_toast_stays_at_the_bottom() {
    let mut harness = start(PlainPage);
    let at = harness.centre("#add").expect("the button");
    harness.send(Input::click(at));
    harness.advance(ms(400));
    assert_eq!(
        harness.attr(".ds-toast", "data-place").as_deref(),
        Some("bottom")
    );
    let toast = harness.rect(".ds-toast").expect("the toast");
    assert!(toast.origin.y.0 > VIEW.height as f32 / 2.0, "{toast:?}");
}

#[test]
fn a_toast_under_a_still_pointer_lets_the_press_through_and_is_not_held() {
    let mut harness = start(UnderPage);
    let at = harness.centre("#add").expect("the button");
    harness.send(Input::click(at));
    harness.advance(ms(400));
    assert_eq!(
        harness.attr(".ds-toast", "data-armed").as_deref(),
        Some("waiting")
    );
    let toast = harness.rect(".ds-toast").expect("the toast");
    assert!(
        toast.origin.x.0 < at.x.0 && at.x.0 < toast.origin.x.0 + toast.size.width.0,
        "the toast {toast:?} is over the pointer {at:?}"
    );
    harness.send(Input::click(at));
    harness.advance(ms(100));
    assert_eq!(harness.text_of(".presses").as_deref(), Some("2"));
    assert_eq!(harness.text_of(".undos").as_deref(), Some("0"));
    harness.advance(ms(6000));
    assert_eq!(harness.count(".ds-toast"), 0, "the hold ran out");
}

#[test]
fn after_a_move_onto_the_toast_it_holds_and_undo_takes_the_press() {
    let mut harness = start(UnderPage);
    let at = harness.centre("#add").expect("the button");
    harness.send(Input::click(at));
    harness.advance(ms(400));
    let action = harness.centre(".ds-toast-action").expect("the action");
    // The first move arms the toast; the second lands on it as a live surface.
    harness.send(Input::pointer_move(action));
    harness.advance(ms(50));
    harness.send(Input::pointer_move(Point {
        x: Px(action.x.0 + 1.0),
        y: action.y,
    }));
    harness.advance(ms(50));
    assert_eq!(
        harness.attr(".ds-toast", "data-armed").as_deref(),
        Some("live")
    );
    harness.advance(ms(6000));
    assert_eq!(harness.count(".ds-toast"), 1, "held under the pointer");
    harness.send(Input::click(action));
    harness.advance(ms(20));
    assert_eq!(harness.text_of(".undos").as_deref(), Some("1"));
    assert_eq!(harness.text_of(".presses").as_deref(), Some("1"));
}
