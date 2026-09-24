//! G5 (mailo Phase B): copy and paste. Ctrl+C on a selection in one field and Ctrl+V into
//! another move the text through the harness's in-memory clipboard, and the app's own
//! `ds_native::clipboard::{write_text, read_text}` reach the same clipboard.

use dioxus::prelude::*;
use ds::{Appearance, Button, ButtonVariant, Ds, InputVariant, Key, Material, Point, TextInput};
use ds_native::clipboard::{ClipboardError, read_text, write_text};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 360,
    height: 260,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn centre(harness: &Harness, selector: &str) -> Point {
    harness
        .centre(selector)
        .unwrap_or_else(|| panic!("{selector} missing:\n{}", harness.html()))
}

/// Two fields, each echoing its value below, and two buttons that copy and paste through the
/// app's own clipboard calls.
#[allow(non_snake_case)]
fn Fields() -> Element {
    let mut from = use_signal(|| "hello quire".to_owned());
    let mut to = use_signal(String::new);
    let mut pasted = use_signal(String::new);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { style: "display:flex; flex-direction:column; gap:12px; width:300px; padding:12px",
                div { id: "from", style: "display:flex",
                    TextInput { variant: InputVariant::Boxed, label: "From", value: from(),
                        oninput: move |value| from.set(value) }
                }
                div { id: "to", style: "display:flex",
                    TextInput { variant: InputVariant::Boxed, label: "To", value: to(),
                        oninput: move |value| to.set(value) }
                }
                div { id: "copy", style: "display:flex",
                    Button { variant: ButtonVariant::Secondary, label: "Copy address",
                        onclick: move |_| { let _ = write_text("ada@example.org"); } }
                }
                div { id: "paste", style: "display:flex",
                    Button { variant: ButtonVariant::Secondary, label: "Paste",
                        onclick: move |_| pasted.set(read_text().unwrap_or_else(|e| format!("error: {e:?}"))) }
                }
            }
            p { class: "to", {to()} }
            p { class: "pasted", {pasted()} }
        }
    }
}

#[test]
fn ctrl_c_on_a_selection_then_ctrl_v_into_a_second_field() {
    let mut harness = Harness::new(Fields, VIEW);
    let from = centre(&harness, "#from input");
    harness.click(from);
    harness.chord(&[Key::Ctrl], Key::Char('a'));
    assert_eq!(
        harness.selected_text("#from input").as_deref(),
        Some("hello quire")
    );
    harness.chord(&[Key::Ctrl], Key::Char('c'));
    assert_eq!(harness.clipboard_text().as_deref(), Some("hello quire"));

    let to = centre(&harness, "#to input");
    harness.click(to);
    harness.chord(&[Key::Ctrl], Key::Char('v'));
    harness.advance(ms(20));
    assert_eq!(harness.text_of(".to").as_deref(), Some("hello quire"));
}

#[test]
fn the_apps_write_and_read_reach_the_same_clipboard() {
    let mut harness = Harness::new(Fields, VIEW);
    let copy = centre(&harness, "#copy .ds-button");
    harness.click(copy);
    assert_eq!(harness.clipboard_text().as_deref(), Some("ada@example.org"));

    harness.set_clipboard_text("from another app");
    let paste = centre(&harness, "#paste .ds-button");
    harness.click(paste);
    assert_eq!(
        harness.text_of(".pasted").as_deref(),
        Some("from another app")
    );
}

#[test]
fn outside_a_document_there_is_no_clipboard() {
    assert_eq!(read_text(), Err(ClipboardError::NoHost));
}
