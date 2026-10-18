//! A click on an actions menu hung over the palette's rows (design/06 §20.2): the menu's entry
//! takes the press and the click, so its pick runs and the row under it is neither picked nor
//! selected, as in the launcher.

use dioxus::prelude::*;
use ds::components::menus::palette::palette_group::PaletteGroup;
use ds::components::menus::palette::palette_group::PaletteRow;
use ds::focus::request::use_focus_request;
use ds::host::measure::Anchor;
use ds::prelude::*;
use ds_harness::{Clock, Harness, HarnessConfig, Input, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 720,
    height: 480,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn rows() -> Vec<PaletteRow<u8>> {
    [
        "Files",
        "Firefox",
        "Fonts",
        "Fractions",
        "Friends",
        "Frames",
    ]
    .into_iter()
    .zip(1u8..)
    .map(|(title, value)| PaletteRow::new(value, title.to_string()))
    .collect()
}

/// Whether `event` is the actions key, Ctrl+K.
fn is_actions(event: &KeyboardEvent) -> bool {
    event.modifiers().ctrl() && event.key() == dioxus::prelude::Key::Character("k".to_string())
}

#[allow(non_snake_case)]
fn Launcher() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut actions = use_signal(|| false);
    let mut row = use_signal(|| None::<Rect>);
    let field = use_focus_request();
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            p { class: "log", {log().join(",")} }
            div { style: "width:600px; height:400px",
                CommandPalette::<u8> {
                    label: "Launch".to_string(),
                    placeholder: "Search".to_string(),
                    query: String::new(),
                    tokens: Vec::new(),
                    groups: vec![PaletteGroup::list("Applications", rows())],
                    empty: "Nothing".to_string(),
                    oninput: move |_| {},
                    onpick: move |value: u8| log.with_mut(|log| log.push(format!("row:{value}"))),
                    onclose: move |()| {},
                    host: CommandPaletteHost::Surface,
                    id: "card".to_string(),
                    focus: field,
                    on_select: move |index: usize| log.with_mut(|log| log.push(format!("select:{index}"))),
                    on_select_rect: move |rect: Rect| row.set(Some(rect)),
                    onkey: move |event: KeyboardEvent| {
                        if is_actions(&event) {
                            event.prevent_default();
                            event.stop_propagation();
                            actions.set(true);
                        }
                    },
                }
            }
            if let (true, Some(rect)) = (actions(), row()) {
                Menu::<u8> {
                    placement: MenuPlacement::Popup,
                    anchor: Anchor::Rect(rect),
                    items: vec![MenuItem::new(9, "Open"), MenuItem::new(10, "Show in Files"), MenuItem::new(11, "Quit")],
                    start: Some(0),
                    onpick: move |value: u8| log.with_mut(|log| log.push(format!("menu:{value}"))),
                    onclose: move |()| {
                        actions.set(false);
                        field.request();
                    },
                }
            }
        }
    }
}

fn opened() -> Harness {
    let mut harness = Harness::new(
        Launcher,
        HarnessConfig::new(VIEW).with_clock(Clock::Virtual),
    );
    harness.advance(ms(300));
    harness.send(Input::chord(&[ShortcutKey::Ctrl], ShortcutKey::Char('k')));
    harness.advance(ms(300));
    assert_eq!(harness.count(".ds-popover.ds-menu"), 1, "the menu is up");
    harness
}

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

#[test]
fn a_click_on_an_entry_over_the_rows_picks_the_entry_and_not_the_row() {
    let mut harness = opened();
    let before = log(&harness);
    let at = harness
        .centre(".ds-popover.ds-menu .ds-menu-item:nth-child(2) .ds-menu-label")
        .expect("a second entry");
    harness.send(Input::click(at));
    harness.advance(ms(300));
    let after = log(&harness);
    let new = after
        .strip_prefix(&before)
        .unwrap_or(&after)
        .trim_start_matches(',');
    assert_eq!(new, "menu:10", "only the entry ran (log: {after})");
}
