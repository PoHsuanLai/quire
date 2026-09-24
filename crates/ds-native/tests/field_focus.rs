//! mailo gaps 2, step 1: a `TextInput`'s `onfocus` and `onblur` on a real Blitz document. A
//! click moves the caret and fires the renderer's own events; the focus seam
//! (`Focus::Controlled`) moves it with no event, and the field reports it itself.

use dioxus::prelude::*;
use ds::{
    Appearance, Button, ButtonVariant, Ds, Focus, InputVariant, Material, TextInput, TextInputKind,
    use_focus_request,
};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 480,
    height: 320,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

/// Two plain fields, `a` and `b`, a password `c` the caller focuses through a request (and on
/// mount), and a button that makes the request. Every focus and blur is logged.
#[allow(non_snake_case)]
fn Fields() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let request = use_focus_request();
    let mut note = move |what: &str| log.with_mut(|log| log.push(what.to_string()));
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { style: "display:flex; flex-direction:column; gap:12px; width:300px; padding:12px",
                div { id: "a", style: "display:flex",
                    TextInput { variant: InputVariant::Boxed, label: "A", value: "", oninput: |_| {},
                        onfocus: move |()| note("focus:a"), onblur: move |()| note("blur:a") }
                }
                div { id: "b", style: "display:flex",
                    TextInput { variant: InputVariant::Boxed, label: "B", value: "", oninput: |_| {},
                        onfocus: move |()| note("focus:b"), onblur: move |()| note("blur:b") }
                }
                div { id: "c", style: "display:flex",
                    TextInput { variant: InputVariant::Boxed, kind: TextInputKind::Password, label: "C", value: "secret",
                        focus: Focus::Controlled(request), oninput: |_| {},
                        onfocus: move |()| note("focus:c"), onblur: move |()| note("blur:c") }
                }
                div { style: "display:flex",
                    Button { variant: ButtonVariant::Secondary, label: "Back to C", onclick: move |_| request.request() }
                }
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

/// The seam focuses `c` on mount and says so; a click on `a` blurs `c` and focuses `a`; a click
/// on `b` blurs `a` and focuses `b`.
#[test]
fn a_click_and_the_seam_both_report_focus_and_a_click_reports_blur() {
    let mut harness = Harness::new(Fields, VIEW);
    harness.advance(ms(100));
    assert_eq!(log(&harness), "focus:c", "the seam focused c on mount");
    assert!(harness.is_focused("#c input"));

    let a = harness.centre("#a input").expect("the field");
    harness.click(a);
    harness.advance(ms(50));
    assert_eq!(log(&harness), "focus:c,blur:c,focus:a");
    assert!(harness.is_focused("#a input"));

    let b = harness.centre("#b input").expect("the field");
    harness.click(b);
    harness.advance(ms(50));
    assert_eq!(log(&harness), "focus:c,blur:c,focus:a,blur:a,focus:b");
}

/// `Focus::Controlled` with the new handlers: a request after the person moved away brings the
/// caret back to `c`, and `c` reports it through `onfocus` although the host moved it silently.
#[test]
fn a_controlled_request_still_focuses_and_reports_it() {
    let mut harness = Harness::new(Fields, VIEW);
    harness.advance(ms(100));
    let b = harness.centre("#b input").expect("the field");
    harness.click(b);
    harness.advance(ms(50));
    let before = log(&harness);
    assert_eq!(before, "focus:c,blur:c,focus:b");

    let button = harness.centre(".ds-button").expect("the request button");
    harness.click(button);
    harness.advance(ms(100));
    assert!(harness.is_focused("#c input"), "{}", harness.html());
    let after = log(&harness);
    let added = after.strip_prefix(&before).unwrap_or(&after);
    assert!(
        added.ends_with(",focus:c"),
        "the request reported c's focus: {added}"
    );
}

/// The password never paints: the input holds the value, the mask holds one dot per character.
#[test]
fn a_password_draws_dots_not_its_value() {
    let harness = Harness::new(Fields, VIEW);
    assert_eq!(
        harness.attr("#c input", "type").as_deref(),
        Some("password")
    );
    assert_eq!(
        harness.text_of("#c .ds-input-mask").as_deref(),
        Some("••••••")
    );
}
