//! A FieldRow's help on request (design/04-COMPONENTS.md, "FieldRow help"), on a real Blitz
//! document: the ⓘ button opens a popover with the label as its title and the help under it;
//! Escape and an outside press close it; only one row's popover is open at a time; the help text
//! is never drawn inline and stays the row's `aria-describedby`.

use dioxus::prelude::*;
use ds::components::fields::field_help::HelpShown;
use ds::components::fields::field_row::FieldRow;
use ds::prelude::*;
use ds_harness::{Clock, Harness, HarnessConfig, Input, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 640,
    height: 400,
    scale_percent: 100,
};

const FIRST: &str = "#first .ds-field-row-about .ds-button";
const SECOND: &str = "#second .ds-field-row-about .ds-button";

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

#[allow(non_snake_case)]
fn Page() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            div { style: "width:420px;margin:20px",
                FieldRow {
                    label: "Wi-Fi",
                    help: Some(TextLine::from("Join known networks.")),
                    help_shown: HelpShown::OnRequest,
                    common: Common { id: Some("first".to_owned()), ..Common::default() },
                    span { "control" }
                }
                // Far enough below that the first row's card does not cover this row's button.
                div { style: "height:160px" }
                FieldRow {
                    label: "Bluetooth",
                    help: Some(TextLine::from("Connect accessories.")),
                    help_shown: HelpShown::OnRequest,
                    common: Common { id: Some("second".to_owned()), ..Common::default() },
                    span { "control" }
                }
            }
        }
    }
}

fn started() -> Harness {
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(200));
    harness
}

fn press(harness: &mut Harness, selector: &str) {
    let at = harness.centre(selector).expect("the button");
    harness.send(Input::click(at));
    harness.advance(ms(400));
}

/// The help popovers open: the card, since the button's own "About" tip is a popover too.
fn popovers(harness: &Harness) -> usize {
    harness.count(".ds-field-help-card")
}

#[test]
fn the_button_is_named_and_the_help_is_not_drawn_inline_but_describes_the_row() {
    let harness = started();
    assert_eq!(
        harness.attr(FIRST, "aria-label").as_deref(),
        Some("About Wi-Fi")
    );
    assert_eq!(
        harness.attr("#first", "aria-describedby").as_deref(),
        Some("first-help")
    );
    assert_eq!(
        harness.text_of("#first-help").as_deref(),
        Some("Join known networks.")
    );
    let help = harness.rect("#first-help").expect("the hidden help");
    assert!(help.size.width.0 <= 1.5, "{help:?}: visually hidden");
    assert_eq!(popovers(&harness), 0);
}

#[test]
fn a_press_opens_the_popover_with_title_and_help_and_a_second_press_closes_it() {
    let mut harness = started();
    press(&mut harness, FIRST);
    assert_eq!(popovers(&harness), 1);
    assert_eq!(
        harness.text_of(".ds-field-help-title").as_deref(),
        Some("Wi-Fi")
    );
    assert_eq!(
        harness.text_of(".ds-field-help-text").as_deref(),
        Some("Join known networks.")
    );
    assert_eq!(
        harness.attr(FIRST, "aria-expanded").as_deref(),
        Some("true")
    );
    press(&mut harness, FIRST);
    assert_eq!(popovers(&harness), 0, "the second press closes it");
}

#[test]
fn escape_and_an_outside_press_close_it() {
    let mut harness = started();
    press(&mut harness, FIRST);
    harness.send(Input::key(ShortcutKey::Escape));
    harness.advance(ms(400));
    assert_eq!(popovers(&harness), 0, "Escape closes it");

    press(&mut harness, FIRST);
    assert_eq!(popovers(&harness), 1);
    let elsewhere = Point {
        x: Px(600.0),
        y: Px(360.0),
    };
    harness.send(Input::click(elsewhere));
    harness.advance(ms(400));
    assert_eq!(popovers(&harness), 0, "an outside press closes it");
}

#[test]
fn only_one_rows_popover_is_open_at_a_time() {
    let mut harness = started();
    press(&mut harness, FIRST);
    assert_eq!(popovers(&harness), 1);
    press(&mut harness, SECOND);
    assert_eq!(
        popovers(&harness),
        0,
        "the press on another row closes the first"
    );
    press(&mut harness, SECOND);
    assert_eq!(popovers(&harness), 1);
    assert_eq!(
        harness.text_of(".ds-field-help-title").as_deref(),
        Some("Bluetooth")
    );
}
