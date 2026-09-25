//! SettingsRow on a real Blitz document (sill FINDINGS Q79): a toggle row's switch flips the
//! device and never runs the row; a press on the row's words runs the row and leaves the switch
//! alone; Enter on the focused row runs it; a disabled row does neither.

use dioxus::prelude::*;
use ds::{
    Appearance, Availability, Ds, Icon, Key, Material, Point, RowTrailing, SettingsRow, Switch,
};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 480,
    height: 240,
    scale_percent: 100,
};

/// Two device rows with switches, the second disabled; the page logs what each heard.
#[allow(non_snake_case)]
fn RowsApp() -> Element {
    let mut on = use_signal(|| Switch::Off);
    let mut log = use_signal(Vec::<String>::new);
    let mut note = move |entry: String| log.with_mut(|log| log.push(entry));
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Popover,
            div { class: "list", style: "width:300px;padding:12px",
                SettingsRow {
                    glyph: Icon::Headphones,
                    title: "Headphones",
                    detail: "Battery 84%",
                    trailing: RowTrailing::Toggle {
                        value: on(),
                        on_toggle: EventHandler::new(move |next: Switch| {
                            on.set(next);
                            note(format!("toggle:{next:?}"));
                        }),
                    },
                    onclick: move |_| note("row:headphones".to_owned()),
                }
                SettingsRow {
                    glyph: Icon::Mouse,
                    title: "Mouse",
                    trailing: RowTrailing::Toggle {
                        value: Switch::Off,
                        on_toggle: EventHandler::new(move |_| note("toggle:mouse".to_owned())),
                    },
                    availability: Availability::Disabled,
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
const FIRST: &str = ".list > .ds-settings-row:first-child";
const SECOND: &str = ".list > .ds-settings-row:last-child";

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

    harness.click(centre(&harness, &format!("{FIRST} .ds-settings-row-title")));
    harness.advance(TICK);
    assert_eq!(
        log(&harness),
        "toggle:On,row:headphones",
        "the words run the row only"
    );

    harness.key(Key::Tab);
    assert!(harness.is_focused(FIRST), "Tab reaches the row");
    harness.key(Key::Enter);
    harness.advance(TICK);
    assert_eq!(log(&harness), "toggle:On,row:headphones,row:headphones");

    harness.click(centre(&harness, &format!("{SECOND} .ds-toggle")));
    harness.click(centre(
        &harness,
        &format!("{SECOND} .ds-settings-row-title"),
    ));
    harness.advance(TICK);
    assert_eq!(
        log(&harness),
        "toggle:On,row:headphones,row:headphones",
        "a disabled row and its switch hear nothing"
    );
}
