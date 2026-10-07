//! A grouped list (design/34-MODERN-LOOK.md section 3.5) on a real Blitz document: rows that open
//! something (a trailing chevron) are still moved among and picked by the keyboard, exactly as the
//! plain list's are; the group is a borderless card of `--row-settings-h` rows; and a selected
//! row is the quiet fill wash, never the accent.

#[path = "support/probe.rs"]
mod probe;

use dioxus::prelude::*;
use ds::base::vocab::RowState;
use ds::components::lists::list::model::ListStyle;
use ds::components::lists::row::size::RowSize;
use ds::prelude::*;
use ds::style::tokens::accent_table::accent_of;
use ds::style::tokens::hex::Hex;
use ds::style::tokens::row_scale::ROW_SCALE;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use probe::{distance, keep, modal, pixels, rect};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 360,
    height: 220,
    scale_percent: 100,
};

const TICK: Duration = Duration::from_millis(50);

const PANES: [&str; 3] = ["Wi-Fi", "Bluetooth", "Sound"];

/// Three chevron rows in a grouped list; the page logs what the list heard.
#[allow(non_snake_case)]
fn Settings() -> Element {
    let mut cursor = use_signal(|| Some("Wi-Fi"));
    let mut log = use_signal(Vec::<String>::new);
    let items: Vec<ListItem<&'static str>> = PANES
        .into_iter()
        .map(|pane| {
            let selection = if cursor() == Some(pane) {
                Selection::Selected
            } else {
                Selection::Unselected
            };
            ListItem::row(
                pane,
                pane,
                rsx! {
                    Row {
                        title: pane,
                        size: RowSize::Settings,
                        leading: RowLeading::Tile(TileFace::Glyph(Icon::Wifi, Hex([0x0a, 0x84, 0xff]))),
                        accessory: Accessory::Chevron,
                        state: RowState { selection, ..RowState::default() },
                    }
                },
            )
        })
        .collect();
    rsx! {
        Ds { appearance: Appearance { theme: Theme::Light, ..Appearance::default() }, material: Material::Window,
            div { style: "width:320px;margin:12px",
                List::<&'static str> {
                    label: "Settings",
                    style: ListStyle::Grouped,
                    items,
                    cursor: cursor(),
                    onselect: move |key: &'static str| {
                        cursor.set(Some(key));
                        log.with_mut(|log| log.push(format!("select:{key}")));
                    },
                    onpick: move |key: &'static str| log.with_mut(|log| log.push(format!("pick:{key}"))),
                }
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

fn harness() -> Harness {
    let mut harness = Harness::new(
        Settings,
        HarnessConfig::new(VIEW).with_clock(Clock::Virtual),
    );
    harness.advance(TICK);
    harness
}

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

fn focus_list(harness: &mut Harness) {
    let at = harness
        .centre(".ds-list")
        .expect("the list is in the document");
    harness.send(Input::click(at));
    harness.advance(TICK);
}

#[test]
fn chevron_rows_move_and_pick_by_keyboard() {
    let mut harness = harness();
    focus_list(&mut harness);
    harness.send(Input::key(ShortcutKey::Down));
    harness.advance(TICK);
    assert_eq!(
        log(&harness),
        "select:Bluetooth",
        "Down moved the cursor one row"
    );
    harness.send(Input::key(ShortcutKey::Enter));
    harness.advance(TICK);
    assert_eq!(
        log(&harness),
        "select:Bluetooth,pick:Bluetooth",
        "Enter picked the row under the cursor"
    );
    harness.send(Input::key(ShortcutKey::Space));
    harness.advance(TICK);
    assert_eq!(
        log(&harness),
        "select:Bluetooth,pick:Bluetooth,pick:Bluetooth",
        "Space picks as Enter does"
    );
}

#[test]
fn a_grouped_row_is_settings_high_and_the_group_is_a_borderless_card() {
    let harness = harness();
    let row = rect(&harness, ".ds-list-item:nth-child(2) > .ds-row");
    let list = rect(&harness, ".ds-list");
    assert_eq!(
        row.size.height.0,
        f32::from(ROW_SCALE.settings_height.0),
        "a grouped row is --row-settings-h"
    );
    assert_eq!(
        row.size.width.0, list.size.width.0,
        "rows run edge to edge: no padding, no outline box"
    );
    assert_eq!(
        list.size.height.0,
        3.0 * f32::from(ROW_SCALE.settings_height.0),
        "three rows and nothing else"
    );
}

#[test]
fn a_selected_grouped_row_is_a_wash_not_the_accent() {
    let mut harness = harness();
    let frame = harness.render().expect("renders");
    keep(&frame, "grouped-list");
    let first = rect(&harness, ".ds-list-item:nth-child(1) > .ds-row");
    let last = rect(&harness, ".ds-list-item:nth-child(3) > .ds-row");
    let selected = modal(&pixels(&frame, first, 2.0));
    let resting = modal(&pixels(&frame, last, 2.0));
    let Hex([r, g, b]) = accent_of(Accent::Blue, Scheme::Light).fill;
    assert!(
        distance(selected, [r, g, b, 255]) > 60,
        "selected row {selected:?} is the accent"
    );
    assert!(
        distance(selected, resting) > 4,
        "selected row {selected:?} is no different from a resting row {resting:?}"
    );
    assert!(
        resting.iter().take(3).all(|c| *c >= 250),
        "the group's ground is white in light: {resting:?}"
    );
}
