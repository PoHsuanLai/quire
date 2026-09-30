//! A toast with a button of the consumer's own (design/30 section 2.9), on a real Blitz document:
//! the button carries its words, pressing it calls the push's handler once and dismisses the
//! toast, a toast left to its hold never calls it, and a later push replaces both the button and
//! its handler.

use dioxus::prelude::*;
use ds::prelude::*;
use ds::root::common::Common;
use ds::stack::toast_hub::{ToastAction, UndoToken};
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 800,
    height: 500,
    scale_percent: 100,
};

static PRESSED: GlobalSignal<Vec<String>> = Signal::global(Vec::new);

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

#[allow(non_snake_case)]
fn Dark() -> Element {
    rsx! {
        Ds { appearance: Appearance { theme: Theme::Dark, ..Appearance::default() }, material: Material::Window, extent: RootExtent::Viewport,
            Pusher {}
        }
    }
}

#[component]
fn Pusher() -> Element {
    let toasts = use_toasts();
    rsx! {
        Button {
            label: "Send",
            common: Common { id: Some("send".to_string()), ..Common::default() },
            onclick: move |_| toasts.push_action(
                "Sent to Travel".into(),
                ToastAction::new("View").with_icon(Icon::ArrowRight),
                EventHandler::new(|()| PRESSED.write().push("view".to_string())),
            ),
        }
        Button {
            label: "Archive",
            common: Common { id: Some("archive".to_string()), ..Common::default() },
            onclick: move |_| toasts.push_undoable(
                "Archived".into(),
                UndoToken(3),
                EventHandler::new(|token: UndoToken| PRESSED.write().push(format!("undo:{}", token.0))),
            ),
        }
    }
}

fn pushed(button: &str) -> Harness {
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.within(|| PRESSED.write().clear());
    let at = harness.centre(button).expect("the button");
    harness.send(Input::click(at));
    harness.advance(ms(400));
    harness
}

fn pressed(harness: &mut Harness) -> Vec<String> {
    harness.within(|| PRESSED.peek().clone())
}

#[test]
fn the_button_carries_the_consumers_words_and_pressing_it_calls_its_handler_once() {
    let mut harness = pushed("#send");
    assert_eq!(
        harness.text_of(".ds-toast-action").as_deref(),
        Some("View"),
        "the action is the consumer's, not Undo"
    );
    assert_eq!(
        harness.count(".ds-toast-action .ds-ic"),
        1,
        "with its glyph"
    );
    let action = harness.centre(".ds-toast-action").expect("the action");
    harness.send(Input::click(action));
    harness.advance(ms(20));
    assert_eq!(pressed(&mut harness), ["view"]);
    assert_eq!(
        harness.attr(".ds-toast", "data-presence").as_deref(),
        Some("leaving"),
        "pressing it dismisses the toast"
    );
    harness.advance(ms(400));
    assert_eq!(harness.count(".ds-toast"), 0);
    assert_eq!(pressed(&mut harness), ["view"], "and it ran once");
}

#[test]
fn a_toast_left_to_its_hold_never_calls_its_handler() {
    let mut harness = pushed("#send");
    // Past the 5 s hold and the slide out.
    harness.advance(ms(6000));
    assert_eq!(harness.count(".ds-toast"), 0, "the hold ran out");
    assert_eq!(pressed(&mut harness), Vec::<String>::new());
}

#[test]
fn a_later_push_replaces_the_button_and_its_handler() {
    let mut harness = pushed("#send");
    let archive = harness.centre("#archive").expect("the second button");
    harness.send(Input::click(archive));
    harness.advance(ms(400));
    assert_eq!(harness.text_of(".ds-toast-action").as_deref(), Some("Undo"));
    let action = harness.centre(".ds-toast-action").expect("the action");
    harness.send(Input::click(action));
    harness.advance(ms(20));
    assert_eq!(
        pressed(&mut harness),
        ["undo:3"],
        "the undo ran; the first push's handler is gone"
    );
}

/// Each scheme's picture paints; it is written only when `QUIRE_SHOTS` names a directory, as
/// `toast-action-<scheme>.png`, for the progress gallery.
#[test]
fn the_action_toast_paints_in_both_schemes() {
    let out = std::env::var("QUIRE_SHOTS").ok();
    for (app, scheme) in [(Page as fn() -> Element, "light"), (Dark, "dark")] {
        let mut harness = Harness::new(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
        let at = harness.centre("#send").expect("the button");
        harness.send(Input::click(at));
        harness.advance(ms(400));
        let shot = harness.render().expect("renders");
        assert_eq!(shot.width(), VIEW.width, "{scheme}");
        if let Some(dir) = &out {
            shot.save(format!("{dir}/toast-action-{scheme}.png"))
                .expect("writes the shot");
        }
    }
}
