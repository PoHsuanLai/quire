//! The toolbar's search item on a real Blitz document (`NSSearchToolbarItem`): a field while the
//! band has room for its minimum width, a magnifier button below that; pressing the button
//! opens the field in place with the caret in it, and the caret leaving it empty folds it back.

use dioxus::prelude::*;
use ds::components::chrome::toolbar::model::{Picked, ToolbarItem, ToolbarRoom};
use ds::components::chrome::toolbar::search::{SearchSeat, ToolbarSearch};
use ds::components::chrome::toolbar::view::Toolbar;
use ds::components::menus::search::view::SearchField;
use ds::prelude::*;
use ds_harness::{Clock, Driver, FocusState, Harness, HarnessConfig, Input, Query, Viewport};
use std::cell::Cell;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 800,
    height: 300,
    scale_percent: 100,
};

/// The field's minimum width: the band needs this and a gap beyond its items to keep it a field.
const MIN: f32 = 200.0;

thread_local! {
    static ROOM: Cell<f32> = const { Cell::new(900.0) };
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

#[allow(non_snake_case)]
fn Page() -> Element {
    let mut query = use_signal(String::new);
    let trailing = vec![
        ToolbarItem::new(1u8, "Share", Icon::Upload),
        ToolbarItem::new(2u8, "Tag", Icon::Tag),
    ];
    let search = ToolbarSearch {
        value: query(),
        min_width: Px(MIN),
        field: Callback::new(move |seat: SearchSeat| {
            rsx! {
                SearchField::<u8> {
                    label: "Search",
                    value: query(),
                    oninput: move |next: String| query.set(next),
                    onpick: |_| {},
                    focus: seat.focus(),
                    onblur: seat.onblur(),
                }
            }
        }),
    };
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            Toolbar::<u8> {
                trailing,
                title: Some(TextLine::from("Mail")),
                room: ToolbarRoom::Fixed(Px(ROOM.with(Cell::get))),
                search: Some(search),
                onpick: |_: Picked<u8>| {},
            }
            button { id: "elsewhere", style: "position:absolute; left:20px; top:200px", "Elsewhere" }
        }
    }
}

fn start(room: f32) -> Harness {
    ROOM.with(|cell| cell.set(room));
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(100));
    harness
}

#[test]
fn with_room_for_its_minimum_the_field_stays_a_field() {
    // 24 padding, 140 title, two items of 40, the field's 200 and a gap of 8.
    let harness = start(452.0);
    assert_eq!(harness.count(".ds-toolbar-search .ds-input"), 1);
    assert_eq!(
        harness.count(".ds-toolbar-trailing > .ds-button"),
        2,
        "no magnifier button, no chevron: both items stay"
    );
}

#[test]
fn below_its_minimum_the_field_collapses_to_a_magnifier_and_no_item_goes_behind_the_chevron_yet() {
    let harness = start(451.0);
    assert_eq!(harness.count(".ds-toolbar-search .ds-input"), 0);
    assert_eq!(
        harness.count(".ds-toolbar-trailing > .ds-button"),
        3,
        "two items and the magnifier"
    );
}

#[test]
fn pressing_the_magnifier_opens_the_field_in_place_with_the_caret_in_it() {
    let mut harness = start(400.0);
    assert_eq!(harness.count(".ds-toolbar-search"), 0);
    let button = harness
        .centre(".ds-toolbar-trailing > .ds-button:last-child")
        .expect("the magnifier");
    harness.send(Input::click(button));
    harness.advance(ms(300));
    assert_eq!(harness.count(".ds-toolbar-search .ds-input"), 1);
    assert_eq!(
        harness.attr(".ds-toolbar-search", "data-seat").as_deref(),
        Some("expanded")
    );
    assert_eq!(
        harness.focus_of(".ds-toolbar-search .ds-input"),
        FocusState::Focused
    );
}

#[test]
fn an_empty_field_folds_back_when_the_caret_leaves_it() {
    let mut harness = start(400.0);
    let button = harness
        .centre(".ds-toolbar-trailing > .ds-button:last-child")
        .expect("the magnifier");
    harness.send(Input::click(button));
    harness.advance(ms(300));
    assert_eq!(harness.count(".ds-toolbar-search .ds-input"), 1);
    let elsewhere = harness.centre("#elsewhere").expect("the button");
    harness.send(Input::click(elsewhere));
    harness.advance(ms(400));
    assert_eq!(harness.count(".ds-toolbar-search .ds-input"), 0, "folded");
}

#[test]
fn a_field_holding_text_stays_open_when_the_caret_leaves_it() {
    let mut harness = start(400.0);
    let button = harness
        .centre(".ds-toolbar-trailing > .ds-button:last-child")
        .expect("the magnifier");
    harness.send(Input::click(button));
    harness.advance(ms(300));
    harness.send(Input::key(ShortcutKey::Char('a')));
    harness.advance(ms(100));
    let elsewhere = harness.centre("#elsewhere").expect("the button");
    harness.send(Input::click(elsewhere));
    harness.advance(ms(400));
    assert_eq!(harness.count(".ds-toolbar-search .ds-input"), 1);
}

#[test]
fn escape_in_an_empty_open_field_folds_it() {
    let mut harness = start(400.0);
    let button = harness
        .centre(".ds-toolbar-trailing > .ds-button:last-child")
        .expect("the magnifier");
    harness.send(Input::click(button));
    harness.advance(ms(300));
    harness.send(Input::key(ShortcutKey::Escape));
    harness.advance(ms(300));
    assert_eq!(harness.count(".ds-toolbar-search .ds-input"), 0);
}
