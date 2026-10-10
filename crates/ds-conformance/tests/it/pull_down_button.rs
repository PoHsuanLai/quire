//! PullDownButton on a real Blitz document: a click or Down opens its menu, a pick fires
//! `onpick` and leaves the button's label as it was, and Escape closes the menu and gives the
//! keyboard back to the button.

use dioxus::prelude::*;
use ds::prelude::*;
use ds_blitz::FocusFallback;
use ds_harness::{Clock, Driver, FocusState, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 420,
    height: 320,
    scale_percent: 100,
};

const BUTTON: &str = ".ds-popup .ds-button";

fn page() -> Element {
    let mut picked = use_signal(|| 0_u8);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            div { style: "padding:24px",
                PullDownButton::<u8> {
                    items: vec![MenuItem::new(1, "New Space"), MenuItem::new(2, "Settings")],
                    onpick: move |value| picked.set(value),
                    title: "Actions".to_string(),
                }
                span { id: "picked", "data-value": "{picked}" }
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

fn settle(harness: &mut Harness) {
    harness.advance(Duration::from_millis(500));
}

fn click_button(harness: &mut Harness) {
    let at = harness.centre(BUTTON).expect("laid out"); // test-only
    harness.send(Input::pointer_move(at));
    harness.send(Input::click(at));
    harness.advance(Duration::from_millis(16));
}

#[test]
fn a_click_opens_the_menu() {
    let mut harness = started();
    assert_eq!(harness.count(".ds-menu"), 0);
    click_button(&mut harness);
    assert_eq!(harness.count(".ds-menu"), 1);
}

#[test]
fn down_opens_the_menu_while_the_button_has_the_keyboard() {
    let mut harness = started();
    harness.send(Input::key(ShortcutKey::Tab));
    settle(&mut harness);
    assert_eq!(harness.focus_of(BUTTON), FocusState::Focused);
    harness.send(Input::key(ShortcutKey::Down));
    settle(&mut harness);
    assert_eq!(harness.count(".ds-menu"), 1);
}

#[test]
fn a_pick_fires_onpick_and_the_label_stays() {
    let mut harness = started();
    let before = harness.text_of(BUTTON);
    click_button(&mut harness);
    harness.send(Input::key(ShortcutKey::Down));
    harness.send(Input::key(ShortcutKey::Enter));
    settle(&mut harness);
    assert_eq!(harness.count(".ds-menu"), 0, "the menu closed");
    assert_eq!(harness.attr("#picked", "data-value").as_deref(), Some("2"));
    assert_eq!(harness.text_of(BUTTON), before, "the label did not change");
    assert!(before.unwrap_or_default().contains("Actions"));
}

#[test]
fn escape_closes_the_menu_and_the_button_has_the_keyboard() {
    let mut harness = started();
    click_button(&mut harness);
    harness.send(Input::key(ShortcutKey::Escape));
    settle(&mut harness);
    assert_eq!(harness.count(".ds-menu"), 0, "the menu closed");
    assert_eq!(harness.focus_of(BUTTON), FocusState::Focused);
}

fn chevron_page() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            PullDownButton::<u8> {
                items: vec![MenuItem::new(1, "New Space")],
                face: PullDownFace::Chevron,
                common: Common { aria_label: Some("Space actions".to_owned()), ..Common::default() },
                onpick: |_| {},
            }
            PullDownButton::<u8> {
                items: vec![MenuItem::new(1, "New Space")],
                face: PullDownFace::Chevron,
                onpick: |_| {},
            }
        }
    }
}

#[test]
fn the_chevron_face_draws_one_glyph_no_text_and_its_name() {
    let mut harness = Harness::new(
        chevron_page,
        HarnessConfig::new(VIEW).with_clock(Clock::Virtual),
    );
    harness.advance(Duration::from_millis(400));
    assert_eq!(
        harness.count(".ds-popup .ds-button .ds-ic"),
        2,
        "one chevron on each"
    );
    assert_eq!(harness.count(".ds-popup .ds-button-label"), 0, "no text");
    assert_eq!(
        harness
            .attr(".ds-popup:nth-child(1) .ds-button", "aria-label")
            .as_deref(),
        Some("Space actions")
    );
    assert_eq!(
        harness
            .attr(".ds-popup:nth-child(2) .ds-button", "aria-label")
            .as_deref(),
        Some("Menu")
    );
}
