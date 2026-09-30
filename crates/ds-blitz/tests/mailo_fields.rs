//! The secure and plain `TextField` on a real Blitz document. A secure field is typed into
//! and read back through its callbacks while its markup never holds it; a plain field is one line
//! of its parent's text.

use dioxus::prelude::*;
use ds::{Appearance, Ds, FieldBezel, FieldKind, Material, ShortcutKey, TextField};
use ds_blitz::{Harness, Viewport};
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
            '\n' => ShortcutKey::Enter,
            c => ShortcutKey::Char(c),
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
                TextField { kind: FieldKind::Secure, label: "App password", value: "",
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
    harness.key(ShortcutKey::Enter);
    harness.advance(ms(30));
    assert_eq!(
        harness.text_of(".changed").as_deref(),
        Some("abc"),
        "Enter commits"
    );
}

/// A plain field inside a 24 px title and a bezeled one beside it.
#[allow(non_snake_case)]
fn Titled() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { id: "title", style: "display:flex; width:300px; font-size:24px; line-height:1.25",
                TextField { bezel: FieldBezel::Plain, label: "Name", value: "Work", oninput: |_| {} }
            }
            div { id: "bezeled", style: "display:flex; width:300px; font-size:24px; line-height:1.25",
                TextField { label: "Name", value: "Work", oninput: |_| {} }
            }
        }
    }
}

#[test]
fn a_plain_field_is_one_line_of_its_parents_text() {
    let harness = Harness::new(Titled, VIEW);
    let plain = harness.rect("#title input").expect("the plain field");
    // The title's own line, 24 x 1.25: no padding, no border, no size of its own.
    assert!((plain.size.height.0 - 30.0).abs() < 0.5, "{plain:?}");
    let bezeled = harness
        .rect("#bezeled .ds-text-field-frame")
        .expect("the bezeled field");
    // A bezeled field keeps its own face whatever the parent's: a Regular control, 22
    // (design/29-SIZING.md).
    assert!((bezeled.size.height.0 - 22.0).abs() < 0.5, "{bezeled:?}");
}
