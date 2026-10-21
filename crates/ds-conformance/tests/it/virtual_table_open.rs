//! `VirtualTable`'s `onopen`: a double click on a row and Command-Down open it (Return
//! stays a pick); a table with no `onopen` lets Command-Down go on. Command-Up is never the table's, so
//! an outer key handler (the app's keymap) hears it, while a plain Up moves the cursor.

use dioxus::prelude::*;
use ds::components::controls::scroller::handle::use_scroller;
use ds::components::lists::table::model::TableColumn;
use ds::components::lists::virtual_table::VirtualTable;
use ds::prelude::*;
use ds_harness::{Clock, Harness, HarnessConfig, Input, Viewport};

const VIEW: Viewport = Viewport {
    width: 400,
    height: 400,
    scale_percent: 100,
};
const ROWS: usize = 100;
/// The row the cursor starts on.
const START: usize = 5;

/// Whether the page gives the table an `onopen`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Listening {
    Yes,
    No,
}

#[allow(non_snake_case)]
fn Heard() -> Element {
    page(Listening::Yes)
}

#[allow(non_snake_case)]
fn Unheard() -> Element {
    page(Listening::No)
}

fn page(listening: Listening) -> Element {
    let scroller = use_scroller();
    let mut cursor = use_signal(|| Some(START));
    let mut opened = use_signal(String::new);
    let mut outer = use_signal(String::new);
    let columns = vec![TableColumn::new(0u8, "Row", Px(80.0))];
    let onopen = match listening {
        Listening::Yes => Some(EventHandler::new(move |row: usize| {
            opened.with_mut(|log| log.push_str(&format!("{row},")));
        })),
        Listening::No => None,
    };
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div {
                style: "height:200px; width:300px; display:flex; flex-direction:column",
                onkeydown: move |event: KeyboardEvent| {
                    outer.with_mut(|log| log.push_str(&format!("{:?},", event.key())));
                },
                VirtualTable::<u8> {
                    label: "Openable",
                    columns,
                    rows: ROWS,
                    cell: Callback::new(|(row, _): (usize, usize)| rsx! { "{row}" }),
                    scroller,
                    on_sort: |_| {},
                    cursor: cursor(),
                    onselect: move |row| cursor.set(Some(row)),
                    onopen,
                    look: TableLook::Striped,
                }
            }
            p { class: "cursor", "{cursor().map_or(-1, |row| row as i64)}" }
            p { class: "opened", "{opened()}" }
            p { class: "outer", "{outer()}" }
        }
    }
}

fn unfocused(listening: Listening) -> Harness {
    let app: fn() -> Element = match listening {
        Listening::Yes => Heard,
        Listening::No => Unheard,
    };
    Harness::new(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual))
}

fn harness(listening: Listening) -> Harness {
    let mut harness = unfocused(listening);
    // A header takes the focus, so the keys reach the table.
    let at = harness.centre(".ds-table-column").expect("laid out");
    harness.send(Input::click(at));
    harness
}

fn opened(harness: &Harness) -> String {
    harness.text_of(".opened").unwrap_or_default()
}

#[test]
fn a_double_click_on_a_row_opens_it_and_a_single_click_does_not() {
    // No header click first: Blitz counts clicks across targets, so it would make the row's first
    // click the second of a pair.
    let mut harness = unfocused(Listening::Yes);
    let at = harness
        .centre(".ds-vtable-item[data-row=\"7\"]")
        .expect("row 7");
    harness.send(Input::click(at));
    assert_eq!(opened(&harness), "");
    harness.send(Input::click(at));
    assert_eq!(opened(&harness), "7,");
}

#[test]
fn command_down_opens_the_cursor_row_and_is_consumed_and_return_does_not_open() {
    let mut harness = harness(Listening::Yes);
    harness.send(Input::key(ShortcutKey::Enter));
    assert_eq!(opened(&harness), "", "Return is a pick, as in Finder");
    harness.send(Input::chord(&[ShortcutKey::Super], ShortcutKey::Down));
    assert_eq!(opened(&harness), "5,");
    assert_eq!(
        harness.text_of(".cursor").as_deref(),
        Some("5"),
        "Command-Down does not move the cursor"
    );
    assert!(
        !harness
            .text_of(".outer")
            .unwrap_or_default()
            .contains("ArrowDown"),
        "the table took Command-Down"
    );
}

#[test]
fn command_down_goes_on_to_the_app_when_the_table_has_no_onopen() {
    let mut harness = harness(Listening::No);
    harness.send(Input::chord(&[ShortcutKey::Super], ShortcutKey::Down));
    assert_eq!(opened(&harness), "");
    assert_eq!(harness.text_of(".cursor").as_deref(), Some("5"));
    assert!(
        harness
            .text_of(".outer")
            .unwrap_or_default()
            .contains("ArrowDown")
    );
}

#[test]
fn command_up_is_the_apps_and_a_plain_up_moves_the_cursor() {
    let mut harness = harness(Listening::Yes);
    harness.send(Input::chord(&[ShortcutKey::Super], ShortcutKey::Up));
    assert_eq!(
        harness.text_of(".cursor").as_deref(),
        Some("5"),
        "Command-Up does not move the cursor"
    );
    assert!(
        harness
            .text_of(".outer")
            .unwrap_or_default()
            .contains("ArrowUp")
    );
    harness.send(Input::key(ShortcutKey::Up));
    assert_eq!(harness.text_of(".cursor").as_deref(), Some("4"));
}
