//! The alert on a real Blitz document: it opens with the keyboard on the default
//! button (the first, or the first that is not destructive); Return presses the default
//! wherever the keyboard is; Escape presses Cancel and a press outside does nothing (a sheet
//! dims nothing and closes by its buttons); Space presses the button that has the keyboard, and
//! Tab moves it; inline it stands inside a 320 px popover and fits, and floating it is centred in
//! the whole window.

use dioxus::prelude::*;
use ds::{
    Alert, AlertButton, AlertRole, Appearance, Ds, Flow, Material, Motion, Point, Px, RootExtent,
    ShortcutKey, TextLine,
};
use ds_native::{Clock, Harness, HarnessConfig, Viewport};
use std::cell::Cell;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 800,
    height: 600,
    scale_percent: 100,
};

static LOG: GlobalSignal<Vec<String>> = Signal::global(Vec::new);
thread_local! {
    static ACTION_ROLE: Cell<AlertRole> = const { Cell::new(AlertRole::Normal) };
    static FLOW: Cell<Flow> = const { Cell::new(Flow::Floating) };
}

#[allow(non_snake_case)]
fn Page() -> Element {
    let flow = FLOW.with(Cell::get);
    let alert = rsx! {
        Alert {
            title: "Turn Bluetooth off?",
            message: Some(TextLine::from("Bluetooth devices such as keyboards and mice will be disconnected.")),
            buttons: vec![
                AlertButton::new("Turn Off", ACTION_ROLE.with(Cell::get), EventHandler::new(|()| LOG.write().push("action".to_owned()))),
                AlertButton::new("Cancel", AlertRole::Cancel, EventHandler::new(|()| LOG.write().push("cancel".to_owned()))),
            ],
            flow,
        }
    };
    rsx! {
        Ds { appearance: Appearance { motion: Motion::Standard, ..Appearance::default() }, material: Material::Window,
            extent: RootExtent::Viewport,
            div { class: "popover", style: "position:absolute; left:440px; top:24px; width:320px; height:300px",
                p { "Bluetooth" }
                if flow == Flow::Inline {
                    {alert.clone()}
                }
            }
            if flow == Flow::Floating {
                {alert}
            }
            p { class: "log", style: "position:absolute; left:0; bottom:0", {LOG().join(",")} }
        }
    }
}

fn start(action: AlertRole, flow: Flow) -> Harness {
    ACTION_ROLE.with(|cell| cell.set(action));
    FLOW.with(|cell| cell.set(flow));
    let config = HarnessConfig::new(VIEW).with_clock(Clock::Virtual);
    let mut harness = Harness::with_config(Page, config);
    harness.within(|| LOG.write().clear());
    // Past the entrance, and past the focus that waits for the document.
    harness.advance(Duration::from_millis(600));
    harness
}

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

/// The action and Cancel buttons: the first and the second in the markup (drawn the other way
/// round, the default on the right).
const ACTION: &str = ".ds-alert-slot:nth-child(1) > .ds-button";
const CANCEL: &str = ".ds-alert-slot:nth-child(2) > .ds-button";

fn press(harness: &mut Harness, key: ShortcutKey) {
    harness.key(key);
    harness.advance(Duration::from_millis(20));
}

#[test]
fn it_opens_with_the_keyboard_on_the_default_button() {
    let harness = start(AlertRole::Normal, Flow::Floating);
    assert!(harness.is_focused(ACTION), "the action is the default");
    let harness = start(AlertRole::Destructive, Flow::Floating);
    assert!(
        harness.is_focused(CANCEL),
        "a destructive action is not the default: Cancel is"
    );
}

#[test]
fn return_presses_the_default_wherever_the_keyboard_is() {
    let mut harness = start(AlertRole::Normal, Flow::Floating);
    press(&mut harness, ShortcutKey::Enter);
    assert_eq!(log(&harness), "action");

    let mut harness = start(AlertRole::Normal, Flow::Floating);
    press(&mut harness, ShortcutKey::Tab);
    assert!(harness.is_focused(CANCEL), "Tab moves to Cancel");
    press(&mut harness, ShortcutKey::Tab);
    assert!(
        harness.is_focused(ACTION),
        "and back: the keyboard stays in the alert"
    );
    press(&mut harness, ShortcutKey::Tab);
    press(&mut harness, ShortcutKey::Enter);
    assert_eq!(
        log(&harness),
        "action",
        "Return is the default, not the focus"
    );

    let mut harness = start(AlertRole::Destructive, Flow::Floating);
    press(&mut harness, ShortcutKey::Enter);
    assert_eq!(log(&harness), "cancel", "a reflexive Return never destroys");
}

#[test]
fn escape_cancels() {
    for flow in [Flow::Floating, Flow::Inline] {
        let mut harness = start(AlertRole::Normal, flow);
        press(&mut harness, ShortcutKey::Escape);
        assert_eq!(log(&harness), "cancel", "{flow:?}");
    }
}

#[test]
fn space_presses_the_button_with_the_keyboard() {
    let mut harness = start(AlertRole::Destructive, Flow::Floating);
    press(&mut harness, ShortcutKey::Space);
    assert_eq!(log(&harness), "cancel");
    let mut harness = start(AlertRole::Normal, Flow::Floating);
    press(&mut harness, ShortcutKey::Space);
    assert_eq!(log(&harness), "action");
}

#[test]
fn a_press_outside_does_nothing_and_a_button_press_is_its_own() {
    for flow in [Flow::Floating, Flow::Inline] {
        let mut harness = start(AlertRole::Normal, flow);
        let panel = harness.rect(".ds-sheet").expect("the panel");
        // Just above the panel: nothing dims and nothing catches the click for the alert.
        harness.click(Point {
            x: panel.origin.x + Px(20.0),
            y: panel.origin.y - Px(8.0),
        });
        harness.advance(Duration::from_millis(20));
        assert_eq!(log(&harness), "", "{flow:?}");

        let mut harness = start(AlertRole::Normal, flow);
        let at = harness.centre(ACTION).expect("the action");
        harness.click(at);
        harness.advance(Duration::from_millis(20));
        assert_eq!(log(&harness), "action", "{flow:?}");
    }
}

#[test]
fn inline_it_fits_inside_the_popover() {
    let harness = start(AlertRole::Normal, Flow::Inline);
    let popover = harness.rect(".popover").expect("the popover");
    let stage = harness.rect(".popover .ds-alert-stage").expect("its stage");
    assert_eq!(stage, popover, "the stage covers the popover exactly");
    let panel = harness.rect(".popover .ds-sheet").expect("the panel");
    let inside = |outer: ds::Rect, inner: ds::Rect| {
        inner.origin.x.0 >= outer.origin.x.0
            && inner.origin.y.0 >= outer.origin.y.0
            && inner.origin.x.0 + inner.size.width.0 <= outer.origin.x.0 + outer.size.width.0
            && inner.origin.y.0 + inner.size.height.0 <= outer.origin.y.0 + outer.size.height.0
    };
    assert!(inside(popover, panel), "{panel:?} in {popover:?}");
    let centre_x = panel.origin.x.0 + panel.size.width.0 / 2.0;
    assert!(
        (centre_x - (popover.origin.x.0 + 160.0)).abs() < 1.0,
        "centred across"
    );
    for button in [ACTION, CANCEL] {
        let rect = harness.rect(button).expect("a button");
        assert!(inside(panel, rect), "{button}: {rect:?} in {panel:?}");
    }
    assert_eq!(
        harness.count(".ds-overlay .ds-alert"),
        0,
        "not in the overlay"
    );
}

#[test]
fn floating_it_is_centred_in_the_window() {
    let harness = start(AlertRole::Normal, Flow::Floating);
    let panel = harness.rect(".ds-sheet").expect("the panel");
    let centre_x = panel.origin.x.0 + panel.size.width.0 / 2.0;
    let centre_y = panel.origin.y.0 + panel.size.height.0 / 2.0;
    assert!((centre_x - 400.0).abs() < 1.0, "{panel:?}");
    assert!((centre_y - 300.0).abs() < 1.0, "{panel:?}");
    assert!((panel.size.width.0 - 340.0).abs() < 0.5, "the narrow sheet");
    assert_eq!(harness.count(".popover .ds-alert"), 0);
}
