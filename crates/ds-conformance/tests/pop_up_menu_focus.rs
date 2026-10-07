//! A pop-up's menu on a real Blitz document, as a host with no focus keeper has it
//! (`FocusFallback::BlitzDefault`, what a shell surface gets): the button has the keyboard again
//! once a pick or Escape has closed the menu, and a press right after the menu opens, before the
//! button has been measured for the menu, still lands on the item that settles under it.

use dioxus::prelude::*;
use ds::components::menus::pop_up_button::PopUpButton;
use ds::prelude::*;
use ds_blitz::FocusFallback;
use ds_harness::{Clock, Driver, FocusState, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 560,
    height: 640,
    scale_percent: 100,
};

const FRAME: Duration = Duration::from_millis(16);

fn page() -> Element {
    let mut chosen = use_signal(|| 1_u8);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            div {
                PopUpButton::<u8> {
                    items: vec![MenuItem::new(1, "IMAP"), MenuItem::new(2, "POP3")],
                    value: Some(chosen()),
                    onpick: move |value| chosen.set(value),
                    title: "Protocol".to_string(),
                }
                span { id: "chosen", "data-value": "{chosen}" }
            }
        }
    }
}

fn started() -> Harness {
    let config = HarnessConfig::new(VIEW)
        .with_clock(Clock::Virtual)
        .with_focus_fallback(FocusFallback::BlitzDefault);
    let mut harness = Harness::new(page, config);
    harness.advance(Duration::from_millis(400));
    harness
}

fn open(harness: &mut Harness) {
    let button = harness.centre(".ds-popup .ds-button").expect("laid out"); // test-only
    harness.send(Input::pointer_move(button));
    harness.send(Input::click(button));
}

fn chosen(harness: &Harness) -> Option<String> {
    harness.attr("#chosen", "data-value")
}

#[test]
fn the_button_has_the_keyboard_after_a_pick_by_keys() {
    let mut harness = started();
    open(&mut harness);
    harness.advance(Duration::from_millis(400));
    harness.send(Input::key(ShortcutKey::Down));
    harness.send(Input::key(ShortcutKey::Enter));
    harness.advance(Duration::from_millis(800));
    assert_eq!(harness.count(".ds-menu"), 0, "the menu closed");
    assert_eq!(chosen(&harness).as_deref(), Some("2"), "the pick landed");
    assert_eq!(
        harness.focus_of(".ds-popup .ds-button"),
        FocusState::Focused
    );
}

#[test]
fn the_button_has_the_keyboard_after_escape() {
    let mut harness = started();
    open(&mut harness);
    harness.advance(Duration::from_millis(400));
    harness.send(Input::key(ShortcutKey::Escape));
    harness.advance(Duration::from_millis(800));
    assert_eq!(harness.count(".ds-menu"), 0, "the menu closed");
    assert_eq!(
        harness.focus_of(".ds-popup .ds-button"),
        FocusState::Focused
    );
}

/// A press `frames` frames after the menu opened, at the place its second item settles.
fn pressed_after(frames: u32) -> Option<String> {
    let mut probe = started();
    open(&mut probe);
    probe.advance(Duration::from_millis(500));
    let item = probe.centre(".ds-menu-item:nth-child(2)").expect("settled"); // test-only

    let mut run = started();
    open(&mut run);
    for _ in 0..frames {
        run.advance(FRAME);
    }
    run.send(Input::pointer_move(item));
    run.send(Input::click(item));
    run.advance(Duration::from_millis(800));
    chosen(&run)
}

#[test]
fn a_press_in_the_first_frames_after_the_menu_opens_chooses_the_item_under_it() {
    for frames in 0..8 {
        assert_eq!(pressed_after(frames).as_deref(), Some("2"), "{frames} in");
    }
}
