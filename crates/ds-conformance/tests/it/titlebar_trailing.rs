//! The titlebar's trailing slot on a real Blitz document: it is drawn at the bar's end, a click
//! on it reaches the app's own control, and a press that travels over it begins no window move
//! (while the same drag on the bar's empty middle still does).

use crate::window_frame_controls::{BAR, Stub, VIEW, at, centre, drag, log, ms, start};
use dioxus::prelude::*;
use ds::prelude::*;
use ds::window::host::use_window_host_provider;
use ds::window::vocab::Maximized;
use ds_harness::{Driver, Input, Query};
use std::rc::Rc;

#[allow(non_snake_case)]
fn Trailed() -> Element {
    let log = use_signal(Vec::<String>::new);
    let mut picked = use_signal(|| 0u32);
    use_window_host_provider(move || {
        Rc::new(Stub {
            log,
            maximized: Maximized::Off,
        })
    });
    let trailing = rsx! {
        button { class: "mode", onclick: move |_| picked += 1, "Mode" }
    };
    rsx! {
        Ds {
            appearance: Appearance::default(),
            material: Material::Window,
            window: WindowFrame::titlebar("Inbox", TrafficLights::Shown).with_trailing(trailing),
            p { class: "log", {log().join(",")} }
            p { class: "picked", "{picked}" }
        }
    }
}

#[test]
fn the_trailing_slot_is_drawn_inside_the_bar_after_the_title() {
    let harness = start(Trailed);
    let bar = harness.html();
    let title = bar.find("ds-titlebar-title").expect("title area");
    let slot = bar.find("ds-titlebar-trailing").expect("trailing slot");
    assert!(title < slot, "{bar}");
    assert!(centre(&harness, ".ds-titlebar-trailing .mode").x.0 > VIEW.width as f32 / 2.0);
}

#[test]
fn a_click_on_the_trailing_control_reaches_it_and_moves_nothing() {
    let mut harness = start(Trailed);
    let mode = centre(&harness, ".mode");
    harness.send(Input::click(mode));
    harness.advance(ms(60));
    assert_eq!(harness.text_of(".picked").as_deref(), Some("1"));
    assert_eq!(log(&harness), "");
}

#[test]
fn a_press_on_the_trailing_slot_does_not_begin_a_move() {
    let mut harness = start(Trailed);
    let from = centre(&harness, ".mode");
    let path = [
        at(from.x.0 - 8.0, from.y.0),
        at(from.x.0 - 40.0, from.y.0 + 3.0),
    ];
    drag(&mut harness, from, &path);
    assert_eq!(log(&harness), "", "{}", harness.html());
    drag(&mut harness, BAR, &[at(300.0, 16.0), at(330.0, 20.0)]);
    assert_eq!(
        log(&harness),
        "move",
        "the empty bar still moves the window"
    );
}
