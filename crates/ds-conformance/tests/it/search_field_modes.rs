//! SearchField's options on a real Blitz document: the top hit is lit and Return picks it, the
//! host reads the lit row and completes it on Tab, Escape takes one step a press in either
//! order, and the card draws the field and its results as one surface.

use dioxus::prelude::*;
use ds::base::geometry::placement::{Align, Side};
use ds::components::menus::search::view::SearchField;
use ds::host::measure::Anchor;
use ds::prelude::*;
use ds_harness::{Clock, Driver, FocusState, Harness, HarnessConfig, Input, Query, Viewport};
use std::cell::Cell;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 800,
    height: 520,
    scale_percent: 100,
};

#[derive(Clone, Copy)]
struct Setup {
    highlight: InitialHighlight,
    escape: EscapeOrder,
    present: SuggestionsPresent,
    floated: bool,
}

thread_local! {
    static SETUP: Cell<Setup> = const { Cell::new(Setup {
        highlight: InitialHighlight::None,
        escape: EscapeOrder::ClosePanelFirst,
        present: SuggestionsPresent::Popup,
        floated: false,
    }) };
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn sections() -> Vec<SuggestionSection<String>> {
    let rows = |names: &[&str]| {
        names
            .iter()
            .map(|name| MenuItem::new((*name).to_owned(), *name))
            .collect()
    };
    vec![
        SuggestionSection::titled("Top hit", rows(&["Invoice from Dana"])),
        SuggestionSection::titled("People", rows(&["Dana Okafor", "Sam Lindqvist"])),
    ]
}

#[allow(non_snake_case)]
fn Page() -> Element {
    let setup = SETUP.with(Cell::get);
    let mut query = use_signal(String::new);
    let mut lit = use_signal(|| None::<String>);
    let mut log = use_signal(Vec::<String>::new);
    let place = setup.floated.then(|| CardPlace {
        anchor: Anchor::Point(Point {
            x: Px(100.0),
            y: Px(40.0),
        }),
        placement: Placement::new(Side::Bottom, Align::Start),
    });
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
          div { style: "position:relative; height:520px",
            div { style: "position:absolute; left:40px; top:40px; width:300px",
                SearchField::<String> {
                    label: "Search",
                    value: query(),
                    suggestions: sections(),
                    highlight: setup.highlight,
                    escape: setup.escape,
                    present: setup.present,
                    place,
                    cursor: SearchCursor::Is(lit()),
                    on_highlight: move |value: Option<String>| lit.set(value),
                    oninput: move |next: String| query.set(next),
                    onpick: move |value: String| log.with_mut(|log| log.push(format!("pick:{value}"))),
                    onkey: move |event: KeyboardEvent| match event.key() {
                        Key::Tab if lit().is_some() => {
                            event.prevent_default();
                            query.set(lit().unwrap_or_default());
                        }
                        Key::Escape => log.with_mut(|log| log.push("window-escape".to_owned())),
                        _ => {}
                    },
                }
            }
            p { id: "log", style: "position:absolute; left:400px; top:10px", {log().join(",")} }
            p { id: "value", style: "position:absolute; left:400px; top:30px", "{query}" }
          }
        }
    }
}

fn start(setup: Setup) -> Harness {
    SETUP.with(|cell| cell.set(setup));
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(100));
    let at = harness.centre(".ds-input").expect("the field");
    harness.send(Input::click(at));
    harness.advance(ms(300));
    harness
}

fn popup() -> Setup {
    SETUP.with(Cell::get)
}

fn press(harness: &mut Harness, key: ShortcutKey) {
    harness.send(Input::key(key));
    harness.advance(ms(80));
}

fn type_text(harness: &mut Harness, text: &str) {
    text.chars()
        .for_each(|c| press(harness, ShortcutKey::Char(c)));
    harness.advance(ms(300));
}

fn lit(harness: &Harness) -> Option<String> {
    harness.text_of(".ds-menu-item[*|data-selected=true] .ds-menu-label")
}

fn text(harness: &Harness, id: &str) -> String {
    harness.text_of(id).unwrap_or_default()
}

fn top_hit() -> Setup {
    Setup {
        highlight: InitialHighlight::TopHit,
        ..popup()
    }
}

#[test]
fn return_picks_the_top_hit() {
    let mut harness = start(top_hit());
    assert_eq!(lit(&harness).as_deref(), Some("Invoice from Dana"));
    press(&mut harness, ShortcutKey::Enter);
    assert_eq!(text(&harness, "#log"), "pick:Invoice from Dana");
}

#[test]
fn the_top_hit_is_lit_again_after_typing() {
    let mut harness = start(top_hit());
    press(&mut harness, ShortcutKey::Down);
    assert_eq!(lit(&harness).as_deref(), Some("Dana Okafor"));
    type_text(&mut harness, "d");
    assert_eq!(lit(&harness).as_deref(), Some("Invoice from Dana"));
}

#[test]
fn nothing_is_lit_without_a_top_hit() {
    let harness = start(Setup {
        highlight: InitialHighlight::None,
        ..popup()
    });
    assert_eq!(lit(&harness), None);
}

#[test]
fn tab_completes_the_row_that_is_lit() {
    let mut harness = start(popup());
    press(&mut harness, ShortcutKey::Down);
    press(&mut harness, ShortcutKey::Down);
    press(&mut harness, ShortcutKey::Tab);
    assert_eq!(text(&harness, "#value"), "Dana Okafor");
}

#[test]
fn the_host_can_set_the_lit_row() {
    let mut harness = start(popup());
    press(&mut harness, ShortcutKey::Up);
    assert_eq!(lit(&harness).as_deref(), Some("Sam Lindqvist"));
    press(&mut harness, ShortcutKey::Enter);
    assert_eq!(text(&harness, "#log"), "pick:Sam Lindqvist");
}

#[test]
fn escape_closes_the_panel_clears_the_text_then_reaches_the_window() {
    let mut harness = start(popup());
    type_text(&mut harness, "dan");
    let steps = |harness: &Harness| {
        (
            harness.count(".ds-menu"),
            text(harness, "#value"),
            text(harness, "#log"),
        )
    };
    assert_eq!(steps(&harness), (1, "dan".to_owned(), String::new()));
    press(&mut harness, ShortcutKey::Escape);
    assert_eq!(steps(&harness), (0, "dan".to_owned(), String::new()));
    press(&mut harness, ShortcutKey::Escape);
    assert_eq!(steps(&harness), (0, String::new(), String::new()));
    press(&mut harness, ShortcutKey::Escape);
    assert_eq!(steps(&harness).2, "window-escape");
}

#[test]
fn escape_clears_first_then_reaches_the_window() {
    let mut harness = start(Setup {
        escape: EscapeOrder::ClearFirst,
        ..popup()
    });
    type_text(&mut harness, "dan");
    assert_eq!(harness.count(".ds-menu"), 1);
    press(&mut harness, ShortcutKey::Escape);
    assert_eq!(text(&harness, "#value"), "");
    assert_eq!(harness.count(".ds-menu"), 0, "the panel goes with the text");
    assert_eq!(text(&harness, "#log"), "");
    press(&mut harness, ShortcutKey::Escape);
    assert_eq!(text(&harness, "#log"), "window-escape");
}

fn card() -> Setup {
    Setup {
        present: SuggestionsPresent::Card,
        ..popup()
    }
}

#[test]
fn a_card_draws_the_field_and_results_as_one_surface() {
    let harness = start(card());
    assert_eq!(harness.count(".ds-popover"), 0, "no second popup");
    assert_eq!(harness.count(".ds-search-card[*|data-surface=own]"), 1);
    assert_eq!(harness.count(".ds-search-card .ds-input"), 1);
    assert_eq!(harness.count(".ds-search-card .ds-menu-item"), 3);
    let card = harness.rect(".ds-search-card").expect("the card");
    let field = harness.rect(".ds-text-field").expect("the field");
    let rows = harness.rect(".ds-search-card-rows").expect("the rows");
    assert!(
        rows.origin.y.0 >= field.origin.y.0 + field.size.height.0,
        "under the field"
    );
    assert!(rows.origin.y.0 + rows.size.height.0 <= card.origin.y.0 + card.size.height.0 + 0.5);
    assert!(field.origin.y.0 >= card.origin.y.0);
}

#[test]
fn a_floated_card_is_one_popover_holding_both() {
    let harness = start(Setup {
        floated: true,
        ..card()
    });
    assert_eq!(harness.count(".ds-popover"), 1);
    assert_eq!(harness.count(".ds-popover .ds-input"), 1);
    assert_eq!(harness.count(".ds-popover .ds-menu-item"), 3);
    assert_eq!(harness.focus_of(".ds-input"), FocusState::Focused);
}

#[test]
fn a_card_picks_with_return_and_closes_its_rows() {
    let mut harness = start(Setup {
        highlight: InitialHighlight::TopHit,
        ..card()
    });
    press(&mut harness, ShortcutKey::Enter);
    assert_eq!(text(&harness, "#log"), "pick:Invoice from Dana");
    assert_eq!(harness.count(".ds-search-card .ds-menu-item"), 0);
}
