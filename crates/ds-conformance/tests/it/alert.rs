//! The alert on a real Blitz document: it opens with the keyboard on the default
//! button (the first, or the first that is not destructive); Return presses the default
//! wherever the keyboard is; Escape presses Cancel (or, when the caller listens for a dismissal,
//! reports that and presses nothing) and a press outside does nothing (a sheet
//! dims nothing and closes by its buttons); Space presses the button that has the keyboard, and
//! Tab moves it; inline it stands inside a 320 px popover and fits, and floating it is centred in
//! the whole window. With `arm_after` it answers nothing (no key, no click, not even Escape) until
//! the delay has run, then everything as above.

use dioxus::prelude::*;
use ds::components::overlays::alert_model::{AlertButton, AlertRole};
use ds::prelude::*;
use ds_harness::{Clock, Harness, HarnessConfig, Input, Viewport};
use std::cell::Cell;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 800,
    height: 600,
    scale_percent: 100,
};

static LOG: GlobalSignal<Vec<String>> = Signal::global(Vec::new);

/// Whether the page asks the alert to report a dismissal.
#[derive(Clone, Copy, PartialEq)]
enum Listening {
    ToButtonsOnly,
    ToDismissal,
}

thread_local! {
    static ACTION_ROLE: Cell<AlertRole> = const { Cell::new(AlertRole::Normal) };
    static FLOW: Cell<Flow> = const { Cell::new(Flow::Floating) };
    static LISTENING: Cell<Listening> = const { Cell::new(Listening::ToButtonsOnly) };
    static ARM_AFTER: Cell<Option<Duration>> = const { Cell::new(None) };
    static SLOT: Cell<bool> = const { Cell::new(false) };
    static BUTTONS: Cell<bool> = const { Cell::new(true) };
}

#[allow(non_snake_case)]
fn Page() -> Element {
    let flow = FLOW.with(Cell::get);
    let on_dismiss = match LISTENING.with(Cell::get) {
        Listening::ToDismissal => Some(EventHandler::new(|()| {
            LOG.write().push("dismissed".to_owned());
        })),
        Listening::ToButtonsOnly => None,
    };
    let alert = rsx! {
        Alert {
            title: "Turn Bluetooth off?",
            message: Some(TextLine::from("Bluetooth devices such as keyboards and mice will be disconnected.")),
            buttons: if BUTTONS.with(Cell::get) { vec![
                AlertButton::new("Turn Off", ACTION_ROLE.with(Cell::get), EventHandler::new(|()| LOG.write().push("action".to_owned()))),
                AlertButton::new("Cancel", AlertRole::Cancel, EventHandler::new(|()| LOG.write().push("cancel".to_owned()))),
            ] } else { Vec::new() },
            on_dismiss,
            arm_after: ARM_AFTER.with(Cell::get),
            flow,
            if SLOT.with(Cell::get) {
                p { class: "slot", "Which device?" }
            }
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
    start_listening(action, flow, Listening::ToButtonsOnly)
}

fn start_listening(action: AlertRole, flow: Flow, listening: Listening) -> Harness {
    start_with(action, flow, listening, None)
}

/// The alert with `arm_after` set, listening for a dismissal.
fn start_armed_after(flow: Flow, delay: Duration) -> Harness {
    start_with(AlertRole::Normal, flow, Listening::ToDismissal, Some(delay))
}

fn start_with(
    action: AlertRole,
    flow: Flow,
    listening: Listening,
    arm_after: Option<Duration>,
) -> Harness {
    ARM_AFTER.with(|cell| cell.set(arm_after));
    SLOT.with(|cell| cell.set(false));
    BUTTONS.with(|cell| cell.set(true));
    ACTION_ROLE.with(|cell| cell.set(action));
    FLOW.with(|cell| cell.set(flow));
    LISTENING.with(|cell| cell.set(listening));
    let config = HarnessConfig::new(VIEW).with_clock(Clock::Virtual);
    let mut harness = Harness::new(Page, config);
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
    harness.send(Input::key(key));
    harness.advance(Duration::from_millis(20));
}

#[test]
fn it_opens_with_the_keyboard_on_the_default_button() {
    let harness = start(AlertRole::Normal, Flow::Floating);
    assert!(harness.focus_of(ACTION), "the action is the default");
    let harness = start(AlertRole::Destructive, Flow::Floating);
    assert!(
        harness.focus_of(CANCEL),
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
    assert!(harness.focus_of(CANCEL), "Tab moves to Cancel");
    press(&mut harness, ShortcutKey::Tab);
    assert!(
        harness.focus_of(ACTION),
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
fn escape_reports_a_dismissal_and_does_not_press_cancel_when_the_caller_listens() {
    for flow in [Flow::Floating, Flow::Inline] {
        let mut harness = start_listening(AlertRole::Normal, flow, Listening::ToDismissal);
        press(&mut harness, ShortcutKey::Escape);
        assert_eq!(log(&harness), "dismissed", "{flow:?}");

        // A button press is still an answer, not a dismissal.
        let mut harness = start_listening(AlertRole::Normal, flow, Listening::ToDismissal);
        press(&mut harness, ShortcutKey::Enter);
        assert_eq!(log(&harness), "action", "{flow:?}");
    }
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
        harness.send(Input::click(Point {
            x: panel.origin.x + Px(20.0),
            y: panel.origin.y - Px(8.0),
        }));
        harness.advance(Duration::from_millis(20));
        assert_eq!(log(&harness), "", "{flow:?}");

        let mut harness = start(AlertRole::Normal, flow);
        let at = harness.centre(ACTION).expect("the action");
        harness.send(Input::click(at));
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
    let inside = |outer: Rect, inner: Rect| {
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
    assert!((panel.size.width.0 - 260.0).abs() < 0.5, "the alert sheet");
    assert_eq!(harness.count(".popover .ds-alert"), 0);
}

#[test]
fn an_alert_with_an_arming_delay_answers_nothing_until_it_arms() {
    for flow in [Flow::Floating, Flow::Inline] {
        let mut harness = start_armed_after(flow, Duration::from_millis(2000));
        for key in [ShortcutKey::Enter, ShortcutKey::Escape, ShortcutKey::Space] {
            press(&mut harness, key);
        }
        let at = harness.centre(ACTION).expect("the action");
        harness.send(Input::click(at));
        harness.advance(Duration::from_millis(20));
        assert_eq!(
            log(&harness),
            "",
            "{flow:?}: nothing answers before it arms"
        );
        assert_eq!(
            harness.count("[*|data-availability=disabled]"),
            2,
            "{flow:?}"
        );

        harness.advance(Duration::from_millis(2000));
        assert_eq!(
            harness.count("[*|data-availability=disabled]"),
            0,
            "{flow:?}"
        );
        assert!(harness.focus_of(ACTION), "{flow:?}: the keyboard arrives");
        press(&mut harness, ShortcutKey::Enter);
        assert_eq!(
            log(&harness),
            "action",
            "{flow:?}: armed, it answers as ever"
        );
    }
}

/// The alert with a content slot, and with `buttons` or none.
fn start_slotted(flow: Flow, buttons: bool, listening: Listening) -> Harness {
    ARM_AFTER.with(|cell| cell.set(None));
    ACTION_ROLE.with(|cell| cell.set(AlertRole::Normal));
    FLOW.with(|cell| cell.set(flow));
    LISTENING.with(|cell| cell.set(listening));
    SLOT.with(|cell| cell.set(true));
    BUTTONS.with(|cell| cell.set(buttons));
    let config = HarnessConfig::new(VIEW).with_clock(Clock::Virtual);
    let mut harness = Harness::new(Page, config);
    harness.within(|| LOG.write().clear());
    harness.advance(Duration::from_millis(600));
    harness
}

#[test]
fn the_slot_stands_between_the_message_and_the_buttons_and_the_keys_still_work() {
    for flow in [Flow::Floating, Flow::Inline] {
        let mut harness = start_slotted(flow, true, Listening::ToButtonsOnly);
        assert_eq!(
            harness.text_of(".ds-alert .slot").as_deref(),
            Some("Which device?")
        );
        let message = harness.centre(".ds-alert-body").expect("the message");
        let slot = harness.centre(".ds-alert .slot").expect("the slot");
        let footer = harness.centre(".ds-alert-footer").expect("the footer");
        assert!(
            message.y < slot.y && slot.y < footer.y,
            "{flow:?}: {}",
            harness.html()
        );
        assert!(harness.focus_of(ACTION), "{flow:?}");
        press(&mut harness, ShortcutKey::Enter);
        press(&mut harness, ShortcutKey::Escape);
        assert_eq!(log(&harness), "action,cancel", "{flow:?}");
    }
}

#[test]
fn an_alert_with_no_buttons_draws_no_footer_and_answers_nothing() {
    let mut harness = start_slotted(Flow::Floating, false, Listening::ToButtonsOnly);
    assert_eq!(harness.count(".ds-alert-footer"), 0, "{}", harness.html());
    assert_eq!(harness.count(".ds-alert .slot"), 1);
    press(&mut harness, ShortcutKey::Enter);
    press(&mut harness, ShortcutKey::Escape);
    assert!(
        log(&harness).is_empty(),
        "no button to press: {}",
        log(&harness)
    );
}
