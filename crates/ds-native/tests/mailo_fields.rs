//! mailo gaps 4, item 3: the new `TextInput` kinds on a real Blitz document. A secret is typed
//! and read back through its callbacks while its markup never holds it; a growing multiline
//! field gains a row per line typed (a fixed one does not); a file field asks the host to pick.

use dioxus::prelude::*;
use ds::{Appearance, Ds, Grow, InputVariant, Key, Material, Rows, TextInput, TextInputKind};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 480,
    height: 360,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// Type `text` into the focused field, one key at a time, letting each render land.
fn type_text(harness: &mut Harness, text: &str) {
    for c in text.chars() {
        let key = match c {
            '\n' => Key::Enter,
            c => Key::Char(c),
        };
        harness.key(key);
        harness.advance(ms(20));
    }
}

/// A secret field whose every input and change is logged (lengths only on screen, the text in
/// a signal the test reads through `.heard`, outside the field).
#[allow(non_snake_case)]
fn Secret() -> Element {
    let mut inputs = use_signal(Vec::<String>::new);
    let mut changes = use_signal(Vec::<String>::new);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { id: "secret", style: "display:flex; width:300px; padding:12px",
                TextInput { variant: InputVariant::Boxed, kind: TextInputKind::Secret, label: "App password", value: "",
                    oninput: move |text: String| inputs.with_mut(|all| all.push(text)),
                    onchange: move |text: String| changes.with_mut(|all| all.push(text)) }
            }
            p { class: "heard", {inputs().join(",")} }
            p { class: "changed", {changes().join(",")} }
        }
    }
}

#[test]
fn a_secret_is_typed_and_heard_but_never_written_into_the_markup() {
    let mut harness = Harness::new(Secret, VIEW);
    let field = "#secret input";
    harness.click(harness.centre(field).expect("the field"));
    harness.advance(ms(30));
    assert!(harness.is_focused(field));
    assert_eq!(harness.attr(field, "value"), None, "no value before typing");

    type_text(&mut harness, "abc");
    assert_eq!(harness.text_of(".heard").as_deref(), Some("a,ab,abc"));
    assert_eq!(harness.attr(field, "value"), None, "no value after typing");
    assert_eq!(
        harness.text_of("#secret .ds-input-mask").as_deref(),
        Some("\u{2022}\u{2022}\u{2022}"),
        "one dot per character"
    );
    let markup = harness.html();
    let field_markup = markup
        .split("id=\"secret\"")
        .nth(1)
        .and_then(|rest| rest.split("class=\"heard\"").next())
        .expect("the field's markup");
    assert!(!field_markup.contains("abc"), "{field_markup}");

    assert_eq!(harness.text_of(".changed").as_deref(), Some(""));
    harness.key(Key::Enter);
    harness.advance(ms(30));
    assert_eq!(
        harness.text_of(".changed").as_deref(),
        Some("abc"),
        "Enter commits"
    );
}

/// Two multiline fields of two rows, `grow` growing to its content and `fixed` not.
#[allow(non_snake_case)]
fn Areas() -> Element {
    let mut grow = use_signal(String::new);
    let mut fixed = use_signal(String::new);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { style: "display:flex; flex-direction:column; gap:12px; width:300px; padding:12px",
                div { id: "grow", style: "display:flex",
                    TextInput { variant: InputVariant::Boxed, kind: TextInputKind::Multiline { rows: Rows(2), grow: Grow::ToContent },
                        label: "Grows", value: grow(), oninput: move |text| grow.set(text) }
                }
                div { id: "fixed", style: "display:flex",
                    TextInput { variant: InputVariant::Boxed, kind: TextInputKind::Multiline { rows: Rows(2), grow: Grow::Fixed },
                        label: "Fixed", value: fixed(), oninput: move |text| fixed.set(text) }
                }
            }
        }
    }
}

#[test]
fn a_growing_field_gains_a_row_per_line_and_a_fixed_one_does_not() {
    let mut harness = Harness::new(Areas, VIEW);
    for (id, grows) in [("#grow textarea", true), ("#fixed textarea", false)] {
        let before = harness.rect(id).expect("the field").size.height.0;
        harness.click(harness.centre(id).expect("the field"));
        harness.advance(ms(30));
        assert!(harness.is_focused(id), "{id}");
        type_text(&mut harness, "a\nb\nc\nd");
        let after = harness.rect(id).expect("the field").size.height.0;
        let rows = harness.attr(id, "rows");
        if grows {
            assert_eq!(rows.as_deref(), Some("4"), "{id}");
            // Two more lines at the field's 13.5 px x 1.55 line: about 42 px taller.
            let grew = after - before;
            assert!((grew - 2.0 * 13.5 * 1.55).abs() < 2.0, "{id}: grew {grew}");
        } else {
            assert_eq!(rows.as_deref(), Some("2"), "{id}");
            assert_eq!(after, before, "{id}");
        }
    }
}

/// A file field whose picks are counted, and a host that answers the second with a name.
#[allow(non_snake_case)]
fn Picker() -> Element {
    let mut picks = use_signal(|| 0u32);
    let name = if picks() >= 2 { "signature.png" } else { "" };
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { id: "file", style: "display:flex; width:320px; padding:12px",
                TextInput { variant: InputVariant::Boxed, kind: TextInputKind::File, label: "Signature image", value: name,
                    placeholder: "No file chosen", oninput: |_| {}, on_pick: move |()| picks += 1 }
            }
            p { class: "picks", "{picks}" }
        }
    }
}

#[test]
fn the_choose_button_and_the_name_both_ask_the_host_to_pick() {
    let mut harness = Harness::new(Picker, VIEW);
    assert_eq!(harness.text_of(".picks").as_deref(), Some("0"));
    assert_eq!(
        harness.text_of("#file .ds-input-placeholder").as_deref(),
        Some("No file chosen")
    );
    harness.click(harness.centre("#file .ds-icon-button").expect("Choose…"));
    harness.advance(ms(30));
    assert_eq!(harness.text_of(".picks").as_deref(), Some("1"));
    harness.click(harness.centre("#file .ds-input").expect("the name"));
    harness.advance(ms(30));
    assert_eq!(harness.text_of(".picks").as_deref(), Some("2"));
    assert_eq!(
        harness.text_of("#file .ds-input").as_deref(),
        Some("signature.png")
    );
    assert_eq!(harness.count("#file .ds-input-placeholder"), 0);
}

/// A bare field inside a 24 px title and a boxed one beside it.
#[allow(non_snake_case)]
fn Titled() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { id: "title", style: "display:flex; width:300px; font-size:24px; line-height:1.25",
                TextInput { variant: ds::FieldFace::Bare, label: "Name", value: "Work", oninput: |_| {} }
            }
            div { id: "boxed", style: "display:flex; width:300px; font-size:24px; line-height:1.25",
                TextInput { variant: InputVariant::Boxed, label: "Name", value: "Work", oninput: |_| {} }
            }
        }
    }
}

#[test]
fn a_bare_field_is_one_line_of_its_parents_text() {
    let harness = Harness::new(Titled, VIEW);
    let bare = harness.rect("#title input").expect("the bare field");
    // The title's own line, 24 x 1.25: no padding, no border, no size of its own.
    assert!((bare.size.height.0 - 30.0).abs() < 0.5, "{bare:?}");
    let boxed = harness.rect("#boxed input").expect("the boxed field");
    // A boxed field keeps its own face whatever the parent's: 13.5 x 1.55 and its padding.
    assert!(
        (boxed.size.height.0 - (13.5 * 1.55 + 16.0)).abs() < 0.5,
        "{boxed:?}"
    );
}
