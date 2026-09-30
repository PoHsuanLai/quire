//! A multi-line `TextField` on a real Blitz document: Enter adds a line and commits nothing, the
//! caret leaving commits the whole text, and `rows` sets the height.

use dioxus::prelude::*;
use ds::components::fields::text_field_model::FieldRows;
use ds::prelude::*;
use ds_harness::{Driver, Harness, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 480,
    height: 480,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// A three-row and a six-row note and a plain field to move the caret to; what is typed and
/// what is committed are written to `.typed` and `.committed`.
#[allow(non_snake_case)]
fn Notes() -> Element {
    let mut typed = use_signal(String::new);
    let mut committed = use_signal(|| "nothing".to_owned());
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { style: "display:flex; flex-direction:column; gap:12px; width:300px; padding:12px",
                div { id: "three", style: "display:flex",
                    TextField { label: "Note", value: typed(), kind: FieldKind::Multiline,
                        oninput: move |next| typed.set(next),
                        onchange: move |text| committed.set(text) }
                }
                div { id: "six", style: "display:flex",
                    TextField { label: "Long note", value: "", kind: FieldKind::Multiline,
                        rows: FieldRows::Six, oninput: |_| {} }
                }
                div { id: "line", style: "display:flex",
                    TextField { label: "Line", value: "", oninput: |_| {} }
                }
            }
            p { class: "typed", {typed().replace('\n', "|")} }
            p { class: "committed", {committed().replace('\n', "|")} }
        }
    }
}

/// Types `text` one key at a time, as the keyboard does.
fn type_text(harness: &mut Harness, text: &str) {
    for ch in text.chars() {
        harness.send(Input::key(ShortcutKey::Char(ch)));
    }
}

#[test]
fn enter_adds_a_line_and_the_caret_leaving_commits_all_of_it() {
    let mut harness = Harness::new(Notes, VIEW);
    harness.advance(ms(100));
    let note = harness.centre("#three textarea").expect("the note");
    harness.send(Input::click(note));
    type_text(&mut harness, "Hi");
    harness.send(Input::key(ShortcutKey::Enter));
    type_text(&mut harness, "there");
    harness.advance(ms(50));
    assert_eq!(harness.text_of(".typed").as_deref(), Some("Hi|there"));
    assert_eq!(
        harness.text_of(".committed").as_deref(),
        Some("nothing"),
        "Enter is a newline, not a commit"
    );

    let line = harness.centre("#line input").expect("the line field");
    harness.send(Input::click(line));
    harness.advance(ms(50));
    assert_eq!(harness.text_of(".committed").as_deref(), Some("Hi|there"));
}

#[test]
fn rows_set_the_height_in_lines() {
    let mut harness = Harness::new(Notes, VIEW);
    harness.advance(ms(100));
    let three = harness.rect("#three textarea").expect("three").size.height;
    let six = harness.rect("#six textarea").expect("six").size.height;
    assert!(
        (six.0 - 2.0 * three.0).abs() < 1.0,
        "six rows {six:?} are twice three rows {three:?}"
    );
}
