//! A `Row`'s inline-edit slot inside a `List` on a real Blitz document: the field takes the
//! keys and presses typed in it, so the list neither moves, jumps nor picks, and the row is not
//! pressed; a row with no field in it still answers every one of those.

use dioxus::prelude::*;
use ds::components::lists::list::model::ListStyle;
use ds::prelude::*;
use ds_harness::{Clock, Driver, FocusState, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 320,
    height: 200,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// Three folders; the second is being renamed. Everything the list and the rows hear is logged.
#[allow(non_snake_case)]
fn Folders() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut name = use_signal(|| "Receipts".to_owned());
    let mut note = move |what: String| log.with_mut(|log| log.push(what));
    let items: Vec<ListItem<u8>> = [(1u8, "Inbox"), (2, "Receipts"), (3, "Travel")]
        .into_iter()
        .map(|(key, title)| {
            let edit = (key == 2).then(|| {
                rsx! {
                    TextField {
                        bezel: FieldBezel::Plain,
                        label: "Rename folder",
                        value: name(),
                        oninput: move |next| name.set(next),
                        focus: FieldFocus::OnMount,
                    }
                }
            });
            let content = rsx! {
                Row {
                    title: title.to_string(),
                    edit,
                    onclick: move |_| note(format!("click:{key}")),
                }
            };
            ListItem::row(key, title, content)
        })
        .collect();
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "width:300px",
                List::<u8> {
                    label: "Folders",
                    items,
                    style: ListStyle::SourceList,
                    cursor: Some(2u8),
                    onselect: move |key| note(format!("select:{key}")),
                    onpick: move |key| note(format!("pick:{key}")),
                }
            }
            p { class: "log", {log().join(",")} }
            p { class: "name", {name()} }
        }
    }
}

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

#[test]
fn the_field_keeps_its_keys_and_presses_from_the_list_and_the_row() {
    let mut harness = Harness::new(Folders, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(200));
    assert_eq!(harness.focus_of(".ds-row-edit input"), FocusState::Focused);
    harness.send(Input::key(ShortcutKey::Char('x')));
    harness.send(Input::key(ShortcutKey::Down));
    harness.send(Input::key(ShortcutKey::Enter));
    harness.advance(ms(50));
    // The caret sits after the name it was built with, as a browser's input does (Blitz fork
    // ca79f436; it used to start at 0, which is what mailo #28's "nvoicei" was).
    assert_eq!(
        harness.text_of(".name").as_deref(),
        Some("Receiptsx"),
        "the letter went to the field"
    );
    assert_eq!(log(&harness), "", "the list heard nothing");
    let field = harness.centre(".ds-row-edit input").expect("the field");
    harness.send(Input::click(field));
    harness.advance(ms(50));
    assert_eq!(
        log(&harness),
        "",
        "a press in the field does not press the row"
    );
}

#[test]
fn a_row_with_no_field_still_answers_a_press_and_only_the_edited_row_says_so() {
    let mut harness = Harness::new(Folders, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(200));
    let first = harness
        .centre(".ds-row .ds-row-title")
        .expect("the first row");
    harness.send(Input::click(first));
    harness.advance(ms(50));
    assert_eq!(log(&harness), "click:1");
    assert_eq!(
        harness
            .attr(".ds-row[*|data-editing]", "data-editing")
            .as_deref(),
        Some("true")
    );
    assert_eq!(harness.count(".ds-row[*|data-editing]"), 1);
}
