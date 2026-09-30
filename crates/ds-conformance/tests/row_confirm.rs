//! A row asking its own question (design/30 section 2.6), on a real Blitz document: the row's
//! words give way to the question, Cancel takes the keyboard, Escape and Cancel put the row back
//! and only the confirming button runs the confirmation; the row is not picked while it asks.

use dioxus::prelude::*;
use ds::components::controls::button_model::ButtonRole;
use ds::components::lists::row::confirm::RowConfirm;
use ds::prelude::*;
use ds_harness::{Clock, Driver, FocusState, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 640,
    height: 400,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

#[component]
fn Folder() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut asking = use_signal(|| false);
    let confirm = asking().then(|| RowConfirm {
        question: "Delete “Receipts” and its 12 messages?".to_string(),
        confirm: "Delete".to_string(),
        role: ButtonRole::Destructive,
        on_confirm: EventHandler::new(move |()| {
            asking.set(false);
            log.with_mut(|log| log.push("deleted".to_string()));
        }),
        on_cancel: EventHandler::new(move |()| {
            asking.set(false);
            log.with_mut(|log| log.push("kept".to_string()));
        }),
    });
    rsx! {
        p { class: "log", {log().join(",")} }
        div { style: "width:400px;margin:60px 0 0 40px",
            Row {
                title: "Receipts",
                confirm,
                action: RowAction::new(Icon::Trash, "Delete folder", EventHandler::new(move |_| asking.set(true))),
                onclick: move |_| log.with_mut(|log| log.push("row".to_string())),
            }
        }
    }
}

#[allow(non_snake_case)]
fn Page() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            Folder {}
        }
    }
}

fn started() -> Harness {
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(200));
    let delete = harness
        .centre(".ds-row-action .ds-button")
        .expect("the action");
    harness.send(Input::click(delete));
    harness.advance(ms(200));
    harness
}

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

#[test]
fn pressing_the_action_makes_the_row_ask_in_place_of_its_words_and_buttons() {
    let harness = started();
    assert_eq!(
        harness.text_of(".ds-row-title").as_deref(),
        Some("Delete “Receipts” and its 12 messages?")
    );
    assert_eq!(harness.count(".ds-row-action"), 0, "the action gives way");
    assert_eq!(harness.count(".ds-row-confirm .ds-button"), 2);
    assert_eq!(
        harness.attr(".ds-row", "data-confirm").as_deref(),
        Some("true")
    );
    assert_eq!(log(&harness), "", "asking picks nothing");
}

#[test]
fn cancel_takes_the_keyboard_and_escape_puts_the_row_back() {
    let mut harness = started();
    assert_eq!(
        harness.focus_of(".ds-row-confirm .ds-button:nth-child(1)"),
        FocusState::Focused,
        "Cancel holds the keyboard"
    );
    harness.send(Input::key(ShortcutKey::Escape));
    harness.advance(ms(100));
    assert_eq!(log(&harness), "kept");
    assert_eq!(
        harness.text_of(".ds-row-title").as_deref(),
        Some("Receipts")
    );
    assert_eq!(harness.count(".ds-row-action"), 1, "the action is back");
}

#[test]
fn only_the_confirming_button_confirms_and_a_press_on_the_row_picks_nothing() {
    let mut harness = started();
    let words = harness.centre(".ds-row-title").expect("the question");
    harness.send(Input::click(words));
    harness.advance(ms(50));
    assert_eq!(log(&harness), "", "the row is not picked while it asks");
    let confirm = harness
        .centre(".ds-row-confirm .ds-button:nth-child(2)")
        .expect("Delete");
    harness.send(Input::click(confirm));
    harness.advance(ms(100));
    assert_eq!(log(&harness), "deleted");
}
