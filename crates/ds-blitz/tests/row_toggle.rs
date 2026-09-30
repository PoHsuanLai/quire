//! A settings-height `Row` on a real Blitz document: a toggle row's switch flips the device and
//! never runs the row; a press on the row's words runs the row and leaves the switch alone; a
//! disabled row does neither.

use dioxus::prelude::*;
use ds::{
    Accessory, Appearance, Availability, Check, Ds, Icon, Material, Point, Row, RowLeading,
    RowSize, RowState, TextLine,
};
use ds_harness::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 480,
    height: 240,
    scale_percent: 100,
};

/// Two device rows with switches, the second disabled; the page logs what each heard.
#[allow(non_snake_case)]
fn RowsApp() -> Element {
    let mut on = use_signal(|| Check::Off);
    let mut log = use_signal(Vec::<String>::new);
    let mut note = move |entry: String| log.with_mut(|log| log.push(entry));
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Popover,
            div { class: "list", style: "width:300px;padding:12px",
                Row {
                    leading: RowLeading::Icon(Icon::Headphones),
                    title: "Headphones",
                    detail: TextLine::from("Battery 84%"),
                    size: RowSize::Settings,
                    accessory: Accessory::Toggle {
                        value: on(),
                        on_toggle: EventHandler::new(move |next: Check| {
                            on.set(next);
                            note(format!("toggle:{next:?}"));
                        }),
                    },
                    onclick: move |_| note("row:headphones".to_owned()),
                }
                Row {
                    leading: RowLeading::Icon(Icon::Mouse),
                    title: "Mouse",
                    size: RowSize::Settings,
                    accessory: Accessory::Toggle {
                        value: Check::Off,
                        on_toggle: EventHandler::new(move |_| note("toggle:mouse".to_owned())),
                    },
                    state: RowState { availability: Availability::Disabled, ..RowState::default() },
                    onclick: move |_| note("row:mouse".to_owned()),
                }
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

fn centre(harness: &Harness, selector: &str) -> Point {
    harness
        .centre(selector)
        .unwrap_or_else(|| panic!("{selector} is not in the document:\n{}", harness.html()))
}

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

const TICK: Duration = Duration::from_millis(50);
const FIRST: &str = ".list > .ds-row:first-child";
const SECOND: &str = ".list > .ds-row:last-child";

#[test]
fn a_toggle_rows_switch_is_its_own_and_the_row_is_the_rows() {
    let mut harness = Harness::new(RowsApp, VIEW);
    harness.advance(TICK);
    harness.click(centre(&harness, &format!("{FIRST} .ds-toggle")));
    harness.advance(TICK);
    assert_eq!(
        log(&harness),
        "toggle:On",
        "the switch flipped, the row did not run"
    );
    assert_eq!(
        harness
            .attr(&format!("{FIRST} .ds-toggle"), "aria-checked")
            .as_deref(),
        Some("true")
    );

    harness.click(centre(&harness, &format!("{FIRST} .ds-row-title")));
    harness.advance(TICK);
    assert_eq!(
        log(&harness),
        "toggle:On,row:headphones",
        "the words run the row only"
    );

    harness.click(centre(&harness, &format!("{SECOND} .ds-toggle")));
    harness.click(centre(&harness, &format!("{SECOND} .ds-row-title")));
    harness.advance(TICK);
    assert_eq!(
        log(&harness),
        "toggle:On,row:headphones",
        "a disabled row and its switch hear nothing"
    );
}
