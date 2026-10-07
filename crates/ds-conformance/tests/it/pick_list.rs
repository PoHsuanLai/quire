//! PickList on a real Blitz document (design/30 section 2.4): a search field over rows in a
//! popover under its control. The caller narrows the rows by the query it hears; Up, Down and
//! Return move and pick; a row that keeps the list open (a label toggle) yields and stays, a row
//! that closes it (Move to, Create) yields and closes; Escape and an outside press close it.

use dioxus::prelude::*;
use ds::components::menus::palette::palette_group::{PaletteGroup, PaletteGroups, PaletteRow};
use ds::host::measure::Anchor;
use ds::prelude::*;
use ds_harness::{Clock, Driver, FocusState, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 720,
    height: 480,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

const LABELS: [&str; 4] = ["Invoices", "Travel", "Family", "Trash bin"];
const CREATE: usize = 99;

/// The labels that contain `query`, each a toggle, and a Create row that closes the list.
fn groups(query: &str, on: &[usize]) -> Vec<PaletteGroup<usize>> {
    let needle = query.to_lowercase();
    let mut rows: Vec<PaletteRow<usize>> = LABELS
        .iter()
        .enumerate()
        .filter(|(_, name)| name.to_lowercase().contains(&needle))
        .map(|(value, name)| PaletteRow {
            accessory: Accessory::Check(if on.contains(&value) {
                Check::On
            } else {
                Check::Off
            }),
            after: AfterPick::KeepOpen,
            ..PaletteRow::new(value, *name)
        })
        .collect();
    if !query.is_empty() {
        rows.push(PaletteRow::new(CREATE, format!("Create “{query}”")));
    }
    vec![PaletteGroup::list("", rows)]
}

#[component]
fn Labels() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut query = use_signal(String::new);
    let mut on = use_signal(Vec::<usize>::new);
    let mut open = use_signal(|| true);
    rsx! {
        p { class: "log", {log().join(",")} }
        span { class: "state", if open() { "open" } else { "closed" } }
        if open() {
            PickList::<usize> {
                anchor: Anchor::Point(Point { x: Px(120.0), y: Px(80.0) }),
                label: "Labels",
                placeholder: "Filter labels",
                query: query(),
                groups: groups(&query(), &on()),
                empty: "No label matches.",
                oninput: move |text: String| query.set(text),
                onpick: move |value: usize| {
                    log.with_mut(|log| log.push(format!("pick:{value}")));
                    if value != CREATE {
                        on.with_mut(|on| match on.iter().position(|at| *at == value) {
                            Some(at) => {
                                on.remove(at);
                            }
                            None => on.push(value),
                        });
                    }
                },
                onclose: move |()| {
                    open.set(false);
                    log.with_mut(|log| log.push("close".to_string()));
                },
            }
        }
    }
}

#[allow(non_snake_case)]
fn Page() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            Labels {}
        }
    }
}

fn started() -> Harness {
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(400));
    harness
}

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

fn type_text(harness: &mut Harness, text: &str) {
    for c in text.chars() {
        harness.send(Input::key(ShortcutKey::Char(c)));
        harness.advance(ms(20));
    }
}

fn titles(harness: &Harness) -> Vec<String> {
    (1..=6)
        .filter_map(|n| {
            harness.text_of(&format!(
                ".ds-pick-list-rows .ds-row:nth-child({n}) .ds-row-title"
            ))
        })
        .collect()
}

#[test]
fn it_opens_with_the_field_holding_the_keyboard_and_the_first_row_selected() {
    let harness = started();
    assert_eq!(harness.count(".ds-pick-list"), 1);
    assert_eq!(
        harness.focus_of(".ds-pick-list input"),
        FocusState::Focused,
        "the field has the keyboard"
    );
    assert_eq!(titles(&harness), LABELS);
    assert_eq!(
        harness
            .attr(".ds-pick-list-rows .ds-row:nth-child(1)", "aria-selected")
            .as_deref(),
        Some("true")
    );
}

#[test]
fn typing_narrows_the_rows_the_caller_gives_and_the_create_row_follows() {
    let mut harness = started();
    type_text(&mut harness, "tra");
    assert_eq!(
        titles(&harness),
        ["Travel", "Trash bin", "Create “tra”"],
        "the caller's narrowing, the query marked in each"
    );
    assert_eq!(harness.count(".ds-pick-list-rows mark"), 3);
}

#[test]
fn a_row_that_keeps_the_list_open_yields_and_stays_until_a_row_that_closes_it() {
    let mut harness = started();
    harness.send(Input::key(ShortcutKey::Down));
    harness.advance(ms(20));
    harness.send(Input::key(ShortcutKey::Enter));
    harness.advance(ms(20));
    assert_eq!(
        log(&harness),
        "pick:1",
        "Down moved to Travel; Return toggled it"
    );
    assert_eq!(
        harness.count(".ds-pick-list"),
        1,
        "and the list is still up"
    );
    assert_eq!(
        harness
            .attr(".ds-pick-list-rows .ds-row:nth-child(2)", "aria-checked")
            .as_deref(),
        Some("true"),
        "the caller's new state is drawn"
    );
    type_text(&mut harness, "x");
    let create = harness
        .centre(".ds-pick-list-rows .ds-row:nth-child(1) .ds-row-title")
        .expect("the Create row");
    harness.send(Input::pointer_move(create));
    harness.send(Input::click(create));
    harness.advance(ms(400));
    assert_eq!(log(&harness), "pick:1,close,pick:99");
    assert_eq!(harness.text_of(".state").as_deref(), Some("closed"));
}

/// A list opened with a query already in its field, as a host that remembers the last one does.
#[allow(non_snake_case)]
fn Prefilled() -> Element {
    let mut query = use_signal(|| "tra".to_string());
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            p { class: "query", {query()} }
            PickList::<usize> {
                anchor: Anchor::Point(Point { x: Px(120.0), y: Px(80.0) }),
                label: "Labels",
                placeholder: "Filter labels",
                query: query(),
                groups: groups(&query(), &[]),
                empty: "No label matches.",
                oninput: move |text: String| query.set(text),
                onpick: |_| {},
                onclose: |()| {},
            }
        }
    }
}

/// The field opens with the caret after the text it holds, so the next key continues the query
/// instead of landing in front of it.
#[test]
fn a_query_already_in_the_field_is_continued_not_prefixed() {
    let mut harness = Harness::new(
        Prefilled,
        HarnessConfig::new(VIEW).with_clock(Clock::Virtual),
    );
    harness.advance(ms(400));
    assert_eq!(
        harness.focus_of(".ds-pick-list input"),
        FocusState::Focused,
        "the field has the keyboard"
    );
    type_text(&mut harness, "v");
    assert_eq!(harness.text_of(".query").as_deref(), Some("trav"));
}

#[allow(non_snake_case)]
fn Nothing() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            PickList::<usize> {
                anchor: Anchor::Point(Point { x: Px(120.0), y: Px(80.0) }),
                label: "Move to",
                placeholder: "Filter folders",
                query: "zzz",
                groups: PaletteGroups::default(),
                empty: "No folder matches.",
                oninput: |_| {},
                onpick: |_| {},
                onclose: |_| {},
            }
        }
    }
}

#[test]
fn nothing_to_list_says_so_and_return_picks_nothing() {
    let mut harness = Harness::new(Nothing, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(400));
    assert_eq!(
        harness.text_of(".ds-pick-list-empty").as_deref(),
        Some("No folder matches.")
    );
    assert_eq!(harness.count(".ds-pick-list-rows .ds-row"), 0);
    harness.send(Input::key(ShortcutKey::Enter));
    harness.advance(ms(100));
    assert_eq!(
        harness.count(".ds-pick-list"),
        1,
        "Return with no row does nothing"
    );
}

#[test]
fn escape_closes_the_list_after_its_fade() {
    let mut harness = started();
    harness.send(Input::key(ShortcutKey::Escape));
    harness.advance(ms(400));
    assert_eq!(log(&harness), "close");
    assert_eq!(harness.count(".ds-pick-list"), 0);
}
