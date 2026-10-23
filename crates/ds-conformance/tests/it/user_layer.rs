//! The cascade layers (ARCHITECTURE.md section 10): the design system is `@layer ds`, a consumer's
//! `AppStyle` sheet is `@layer app`, the chosen look is `@layer look`, and the person's own styles
//! are unlayered. A one-class user rule beats quire's own attribute-qualified rule, and a
//! one-class app rule beats it too, a look beats a more specific app rule, and a user rule beats
//! a look or an app rule that is more specific: specificity only ever sorts rules inside one
//! layer.

use dioxus::prelude::*;
use ds::prelude::*;
use ds::style::kit::{LookStyle, Styles, UserStyle};
use ds_harness::{Clock, Harness, HarnessConfig, Part, Srgba, Viewport};
use std::cell::Cell;

const VIEW: Viewport = Viewport {
    width: 200,
    height: 100,
    scale_percent: 100,
};

thread_local! {
    static SHEETS: Cell<(&'static str, &'static str, &'static str)> =
        const { Cell::new(("", "", "")) };
}

#[allow(non_snake_case)]
fn Desk() -> Element {
    let (app, look, user) = SHEETS.get();
    let styles = use_signal(|| Styles {
        look: LookStyle(look.to_owned()),
        desktop: UserStyle(user.to_owned()),
        ..Styles::default()
    });
    rsx! {
        Ds {
            appearance: Appearance::default(),
            material: Material::Window,
            sheet: Some(ds_shell::stylesheet()),
            styles,
            if !app.is_empty() {
                AppStyle { css: app }
            }
            Button { label: "Go", onclick: |_| {} }
        }
    }
}

/// The button's painted fill with `app` and `user` as the two sheets.
fn fill(app: &'static str, user: &'static str) -> [u8; 4] {
    fill_with_look(app, "", user)
}

/// The same with a look between them.
fn fill_with_look(app: &'static str, look: &'static str, user: &'static str) -> [u8; 4] {
    SHEETS.set((app, look, user));
    let harness = Harness::new(Desk, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    let Srgba(c) = harness
        .fill_of(".ds-button", Part::Element)
        .expect("button fill");
    c.map(|v| (v * 255.0).round() as u8)
}

const GREEN: [u8; 4] = [0, 255, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];

#[test]
fn quires_own_rule_is_more_specific_than_the_probe_rules() {
    assert_ne!(fill("", ""), GREEN);
    assert_ne!(fill("", ""), BLUE);
}

#[test]
fn a_low_specificity_user_rule_beats_quires_higher_specificity_rule() {
    assert_eq!(fill("", ".ds-button { background: rgb(0,255,0) }"), GREEN);
}

#[test]
fn a_low_specificity_app_rule_beats_quires_higher_specificity_rule() {
    assert_eq!(fill(".ds-button { background: rgb(0,0,255) }", ""), BLUE);
}

#[test]
fn a_user_rule_beats_a_more_specific_app_rule() {
    assert_eq!(
        fill(
            ".ds .ds-button.ds-button[data-variant] { background: rgb(0,0,255) }",
            ".ds-button { background: rgb(0,255,0) }"
        ),
        GREEN
    );
}

#[test]
fn a_look_beats_a_more_specific_app_rule_and_a_user_rule_beats_the_look() {
    let app = ".ds .ds-button.ds-button[data-variant] { background: rgb(0,0,255) }";
    let look = ".ds-button { background: rgb(0,255,0) }";
    assert_eq!(fill_with_look(app, look, ""), GREEN);
    assert_eq!(
        fill_with_look("", look, ".ds-button { background: rgb(0,0,255) }"),
        BLUE
    );
}
