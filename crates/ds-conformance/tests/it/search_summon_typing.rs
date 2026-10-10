//! A search card summoned by a key and typed into at once (Spotlight, mail's ⌘K): the field is
//! asked for by selector with its text selected, and every key typed from the frame the field
//! first reads focused lands in order. A caret moved after the first key once turned "invoice"
//! into "nvoicei".

use dioxus::prelude::*;
use ds::components::menus::search::view::SearchField;
use ds::focus::select::Select;
use ds::focus::selector::focus_by_selector;
use ds::prelude::*;
use ds_harness::{
    Clock, Driver, FocusState, Harness, HarnessConfig, Input, Query, Stepped, Viewport,
};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 800,
    height: 520,
    scale_percent: 100,
};

const FIELD: &str = ".ds-search-card input";

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn sections() -> Vec<SuggestionSection<String>> {
    vec![SuggestionSection::titled(
        "Recents",
        vec![MenuItem::new("Compose".to_owned(), "Compose")],
    )]
}

/// A button that summons the card as mail's ⌘K does: the query cleared, the card shown, and the
/// field asked for by selector with its text selected.
#[allow(non_snake_case)]
fn Page() -> Element {
    let mut query = use_signal(String::new);
    let mut shown = use_signal(|| false);
    let mut in_a_field = use_signal(|| false);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "position:relative; height:520px",
                Button { label: "Summon",
                    onclick: move |_| {
                        query.set(String::new());
                        shown.set(true);
                        spawn(async move {
                            let _ = focus_by_selector(FIELD, Select::All).await;
                        });
                    }
                }
                if shown() {
                    div { style: "position:absolute; left:40px; top:60px; width:400px",
                        SearchField::<String> {
                            label: "Search",
                            value: query(),
                            suggestions: sections(),
                            open: Panel::Shown,
                            ends: Ends::Stop,
                            present: SuggestionsPresent::Card,
                            oninput: move |next: String| query.set(next),
                            onfocus: move |()| in_a_field.set(true),
                            onpick: move |_: String| {},
                        }
                    }
                }
                p { id: "value", style: "position:absolute; left:500px; top:10px", "{query}" }
                p { id: "in-field", style: "position:absolute; left:500px; top:30px", "{in_a_field}" }
            }
        }
    }
}

/// Click Summon, then step frame by frame until the field reads focused.
fn summoned() -> Harness {
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(100));
    let at = harness.centre(".ds-button").expect("the summon button");
    harness.send(Input::click(at));
    let mut frames = 0;
    while harness.focus_of(FIELD) != FocusState::Focused {
        if harness.step_frame() == Stepped::Idle {
            harness.advance(ms(1));
        }
        frames += 1;
        assert!(frames < 120, "the field never took the keyboard");
    }
    harness
}

fn typed(harness: &mut Harness) -> String {
    harness.advance(ms(300));
    harness.text_of("#value").unwrap_or_default()
}

#[test]
fn keys_typed_the_moment_the_field_is_focused_land_in_order() {
    let mut harness = summoned();
    "invoice"
        .chars()
        .for_each(|c| harness.send(Input::key(ShortcutKey::Char(c))));
    assert_eq!(typed(&mut harness), "invoice");
}

#[test]
fn keys_typed_a_frame_apart_from_the_focus_on_land_in_order() {
    let mut harness = summoned();
    "invoice".chars().for_each(|c| {
        harness.send(Input::key(ShortcutKey::Char(c)));
        harness.step_frame();
    });
    assert_eq!(typed(&mut harness), "invoice");
}

#[test]
fn keys_typed_a_few_milliseconds_apart_land_in_order() {
    let mut harness = summoned();
    "invoice".chars().for_each(|c| {
        harness.send(Input::key(ShortcutKey::Char(c)));
        harness.advance(ms(4));
    });
    assert_eq!(typed(&mut harness), "invoice");
}
