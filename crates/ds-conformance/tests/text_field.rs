//! TextField on a real Blitz document: the ring is drawn from `data-focus` while the input has the
//! caret (Blitz's `:focus-within` is always false), a search field's clear button appears with text
//! and empties it, a rejected value writes its message and `aria-invalid`, and a busy field takes
//! no typing and shows a spinner.

use dioxus::prelude::*;
use ds::components::fields::text_field_model::Invalid;
use ds::motion::detail::stamp::EventStamp;
use ds::prelude::*;
use ds_harness::{Driver, FocusState, Harness, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 360,
    height: 240,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

#[allow(non_snake_case)]
fn Page() -> Element {
    let mut name = use_signal(String::new);
    let mut query = use_signal(|| "invoice".to_owned());
    let mut busy = use_signal(String::new);
    let rejected = Validity::Invalid(Invalid {
        message: TextLine::from("Not a valid address."),
        stamp: EventStamp(1),
    });
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "display:flex; flex-direction:column; gap:12px; width:300px; padding:12px",
                div { id: "name",
                    TextField { label: "Name", value: name(), oninput: move |next| name.set(next) }
                }
                div { id: "other",
                    TextField { label: "Other", value: "", oninput: |_| {} }
                }
                div { id: "search",
                    TextField { label: "Search", value: query(), kind: FieldKind::Search, oninput: move |next| query.set(next) }
                }
                div { id: "bad",
                    TextField { label: "Email", value: "dana@", validity: rejected, help: TextLine::from("We never share it."), oninput: |_| {} }
                }
                div { id: "busy",
                    TextField { label: "Checking", value: busy(), availability: Availability::Busy, oninput: move |next| busy.set(next) }
                }
            }
        }
    }
}

fn ring(harness: &Harness, id: &str) -> Option<String> {
    harness.attr(&format!("#{id} .ds-text-field"), "data-focus")
}

#[test]
fn the_ring_follows_the_caret() {
    let mut harness = Harness::new(Page, VIEW);
    harness.advance(ms(50));
    assert_eq!(ring(&harness, "name"), None);
    harness.send(Input::click(
        harness.centre("#name input").expect("the field"),
    ));
    harness.advance(ms(30));
    assert_eq!(ring(&harness, "name").as_deref(), Some("ring"));
    harness.send(Input::click(
        harness.centre("#other input").expect("the other field"),
    ));
    harness.advance(ms(30));
    assert_eq!(ring(&harness, "name"), None, "the caret left it");
    assert_eq!(ring(&harness, "other").as_deref(), Some("ring"));
}

#[test]
fn a_search_field_clears_from_its_button_and_keeps_the_caret() {
    let mut harness = Harness::new(Page, VIEW);
    harness.advance(ms(50));
    assert_eq!(
        harness.attr("#search input", "value").as_deref(),
        Some("invoice")
    );
    let clear = harness
        .centre("#search .ds-button")
        .expect("a clear button");
    harness.send(Input::click(clear));
    harness.advance(ms(80));
    assert_eq!(harness.attr("#search input", "value").as_deref(), Some(""));
    assert_eq!(
        harness.count("#search .ds-button"),
        0,
        "nothing left to clear"
    );
    assert_eq!(
        harness.focus_of("#search input"),
        FocusState::Focused,
        "the caret is back in the field"
    );
}

#[test]
fn a_rejected_value_says_why_and_a_busy_field_takes_no_typing() {
    let mut harness = Harness::new(Page, VIEW);
    harness.advance(ms(50));
    assert_eq!(
        harness
            .attr("#bad .ds-text-field", "data-validity")
            .as_deref(),
        Some("invalid")
    );
    assert_eq!(
        harness.attr("#bad input", "aria-invalid").as_deref(),
        Some("true")
    );
    assert_eq!(
        harness.text_of("#bad .ds-text-field-help").as_deref(),
        Some("Not a valid address."),
        "the message replaces the help"
    );
    assert_eq!(
        harness.count("#busy .ds-progress"),
        1,
        "a spinner after the text"
    );
    harness.send(Input::click(
        harness.centre("#busy input").expect("the busy field"),
    ));
    for c in "abc".chars() {
        harness.send(Input::key(ShortcutKey::Char(c)));
    }
    harness.advance(ms(30));
    assert_eq!(harness.attr("#busy input", "value").as_deref(), Some(""));
}
