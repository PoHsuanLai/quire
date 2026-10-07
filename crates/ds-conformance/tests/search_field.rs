//! SearchField on a real Blitz document: its suggestions panel is owned by the field. A press on
//! the field is inside the panel, the keyboard never leaves the field, Up and Down move the
//! highlight, Enter picks, Escape closes the panel first and clears the field second, a press
//! elsewhere closes it, and the panel is placed under the field at its width, flipping above when
//! there is no room.

use dioxus::prelude::*;
use ds::components::menus::item::text::Marks;
use ds::components::menus::search::view::SearchField;
use ds::prelude::*;
use ds_harness::{Clock, Driver, FocusState, Harness, HarnessConfig, Input, Query, Viewport};
use std::cell::Cell;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 720,
    height: 480,
    scale_percent: 100,
};

thread_local! {
    /// Where the field sits, from the top of the page.
    static TOP: Cell<u32> = const { Cell::new(40) };
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

const PEOPLE: [&str; 3] = ["Dana Okafor", "Sam Lindqvist", "Priya Raman"];

/// The suggestions: a top hit with a second line and marks, then the people.
fn sections(query: &str) -> Vec<SuggestionSection<String>> {
    let hit = MenuItem::new("hit".to_owned(), "Invoice from Dana")
        .with_marks(Marks::of_query("Invoice from Dana", query))
        .with_subtitle("Re: March invoice");
    let people = PEOPLE
        .iter()
        .map(|name| MenuItem::new((*name).to_owned(), *name))
        .collect();
    vec![
        SuggestionSection::titled("Top hit", vec![hit]),
        SuggestionSection::titled("People", people),
    ]
}

#[allow(non_snake_case)]
fn Page() -> Element {
    let mut query = use_signal(String::new);
    let mut log = use_signal(Vec::<String>::new);
    let top = TOP.with(Cell::get);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
          div { style: "position:relative; height:480px",
            div { style: "position:absolute; left:40px; top:{top}px; width:300px",
                SearchField::<String> {
                    label: "Search",
                    value: query(),
                    placeholder: "Search mail",
                    suggestions: sections(&query()),
                    oninput: move |next: String| query.set(next),
                    onpick: move |value: String| log.with_mut(|log| log.push(format!("pick:{value}"))),
                    onsubmit: move |text: String| log.with_mut(|log| log.push(format!("submit:{text}"))),
                }
            }
            button { id: "elsewhere", style: "position:absolute; left:500px; top:400px", "Elsewhere" }
            p { id: "log", style: "position:absolute; left:400px; top:10px", {log().join(",")} }
            p { id: "value", style: "position:absolute; left:400px; top:30px", "{query}" }
          }
        }
    }
}

fn start() -> Harness {
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(100));
    harness
}

fn focus_field(harness: &mut Harness) {
    let at = harness.centre(".ds-input").expect("the field");
    harness.send(Input::click(at));
    harness.advance(ms(300));
}

fn panels(harness: &Harness) -> usize {
    harness.count(".ds-menu")
}

fn log(harness: &Harness) -> String {
    harness.text_of("#log").unwrap_or_default()
}

/// The highlighted row's title: a rich row's title line, else a plain row's label.
fn highlighted(harness: &Harness) -> Option<String> {
    let row = ".ds-menu-item[*|data-selected=true]";
    harness
        .text_of(&format!("{row} .ds-menu-title"))
        .or_else(|| harness.text_of(&format!("{row} .ds-menu-label")))
}

fn press_key(harness: &mut Harness, key: ShortcutKey) {
    harness.send(Input::key(key));
    harness.advance(ms(60));
}

fn type_text(harness: &mut Harness, text: &str) {
    for c in text.chars() {
        press_key(harness, ShortcutKey::Char(c));
    }
    harness.advance(ms(300));
}

#[test]
fn the_panel_opens_with_the_caret_and_lists_sections_with_headers() {
    let mut harness = start();
    assert_eq!(panels(&harness), 0, "closed until the caret arrives");
    focus_field(&mut harness);
    assert_eq!(panels(&harness), 1);
    assert_eq!(harness.count(".ds-menu-header"), 2);
    assert_eq!(harness.count(".ds-menu-item"), 4);
    assert_eq!(
        harness.text_of(".ds-menu-subtitle").as_deref(),
        Some("Re: March invoice")
    );
}

#[test]
fn a_press_on_the_field_leaves_the_panel_open_and_the_keyboard_in_the_field() {
    let mut harness = start();
    focus_field(&mut harness);
    assert_eq!(panels(&harness), 1);
    let at = harness.centre(".ds-input").expect("the field");
    harness.send(Input::click(at));
    harness.advance(ms(400));
    assert_eq!(panels(&harness), 1, "the press on the field is inside");
    assert_eq!(harness.focus_of(".ds-input"), FocusState::Focused);
}

#[test]
fn a_panel_that_opens_mid_press_does_not_take_the_keyboard() {
    let mut harness = start();
    let at = harness.centre(".ds-input").expect("the field");
    harness.send(Input::pointer_down(at));
    harness.advance(ms(300));
    assert_eq!(
        panels(&harness),
        1,
        "opened by the press that focused the field"
    );
    assert_eq!(harness.focus_of(".ds-input"), FocusState::Focused);
    harness.send(Input::pointer_up(at));
    harness.advance(ms(300));
    assert_eq!(panels(&harness), 1);
    assert_eq!(harness.focus_of(".ds-input"), FocusState::Focused);
}

#[test]
fn the_arrows_move_the_highlight_and_the_keyboard_stays_in_the_field() {
    let mut harness = start();
    focus_field(&mut harness);
    assert_eq!(
        highlighted(&harness),
        None,
        "nothing is highlighted at first"
    );
    press_key(&mut harness, ShortcutKey::Down);
    assert_eq!(highlighted(&harness).as_deref(), Some("Invoice from Dana"));
    press_key(&mut harness, ShortcutKey::Down);
    press_key(&mut harness, ShortcutKey::Down);
    assert_eq!(highlighted(&harness).as_deref(), Some("Sam Lindqvist"));
    press_key(&mut harness, ShortcutKey::Up);
    assert_eq!(highlighted(&harness).as_deref(), Some("Dana Okafor"));
    assert_eq!(harness.focus_of(".ds-input"), FocusState::Focused);
    assert_eq!(panels(&harness), 1);
}

#[test]
fn enter_picks_the_highlighted_row_and_submits_when_none_is() {
    let mut harness = start();
    focus_field(&mut harness);
    press_key(&mut harness, ShortcutKey::Down);
    press_key(&mut harness, ShortcutKey::Down);
    press_key(&mut harness, ShortcutKey::Enter);
    harness.advance(ms(400));
    assert_eq!(log(&harness), "pick:Dana Okafor");
    assert_eq!(panels(&harness), 0, "a pick closes the panel");

    type_text(&mut harness, "x");
    press_key(&mut harness, ShortcutKey::Enter);
    assert_eq!(log(&harness), "pick:Dana Okafor,submit:x");
}

#[test]
fn escape_closes_the_panel_first_and_clears_the_field_second() {
    let mut harness = start();
    focus_field(&mut harness);
    type_text(&mut harness, "dan");
    assert_eq!(harness.text_of("#value").as_deref(), Some("dan"));
    assert_eq!(panels(&harness), 1);
    press_key(&mut harness, ShortcutKey::Escape);
    harness.advance(ms(300));
    assert_eq!(panels(&harness), 0, "the first Escape closes the panel");
    assert_eq!(
        harness.text_of("#value").as_deref(),
        Some("dan"),
        "and leaves the text"
    );
    press_key(&mut harness, ShortcutKey::Escape);
    assert_eq!(
        harness.text_of("#value").as_deref(),
        Some(""),
        "the second clears the field"
    );
    assert_eq!(harness.focus_of(".ds-input"), FocusState::Focused);
}

#[test]
fn a_press_elsewhere_closes_the_panel() {
    let mut harness = start();
    focus_field(&mut harness);
    assert_eq!(panels(&harness), 1);
    let at = harness.centre("#elsewhere").expect("the button");
    harness.send(Input::click(at));
    harness.advance(ms(500));
    assert_eq!(panels(&harness), 0);
}

#[test]
fn tab_leaves_the_field_and_the_panel_goes_with_it() {
    let mut harness = start();
    focus_field(&mut harness);
    press_key(&mut harness, ShortcutKey::Tab);
    harness.advance(ms(400));
    assert_ne!(harness.focus_of(".ds-input"), FocusState::Focused);
    assert_eq!(panels(&harness), 0);
}

#[test]
fn the_panel_hangs_under_the_field_at_its_width() {
    let mut harness = start();
    focus_field(&mut harness);
    let field = harness.rect(".ds-text-field").expect("the field");
    let panel = harness.rect(".ds-menu").expect("the panel");
    assert!(
        panel.origin.y.0 >= field.origin.y.0 + field.size.height.0,
        "under it: {panel:?} {field:?}"
    );
    assert!(
        (panel.size.width.0 - field.size.width.0).abs() < 1.0,
        "the field's width: {panel:?} {field:?}"
    );
}

#[test]
fn the_panel_flips_above_the_field_when_there_is_no_room_below() {
    TOP.with(|top| top.set(440));
    let mut harness = start();
    focus_field(&mut harness);
    TOP.with(|top| top.set(40));
    let field = harness.rect(".ds-text-field").expect("the field");
    let panel = harness.rect(".ds-menu").expect("the panel");
    assert!(
        panel.origin.y.0 + panel.size.height.0 <= field.origin.y.0 + 1.0,
        "above it: {panel:?} {field:?}"
    );
}
