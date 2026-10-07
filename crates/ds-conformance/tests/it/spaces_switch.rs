//! Switching Space through the kit on a real Blitz document: Command and a digit switches, the
//! frame takes the Space's look, the place the Space was left at comes back, the slide direction
//! is exposed, and a dot click does not switch while a part of a Space's menu is open.

use crate::support::spaces_page::Three;
use ds::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 420,
    height: 360,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn harness() -> Harness {
    let mut harness = Harness::new(Three, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(50));
    let app = harness.centre(".app").expect("the app is laid out");
    harness.send(Input::click(app));
    harness.advance(ms(50));
    harness
}

/// Click the app's own padding: the keyboard goes back to it after a button took it.
fn focus_app(harness: &mut Harness) {
    let app = harness.rect(".app").expect("the app");
    let at = Point {
        x: Px(app.origin.x.0 + 4.0),
        y: Px(app.origin.y.0 + 4.0),
    };
    harness.send(Input::click(at));
    harness.advance(ms(50));
}

fn command(harness: &mut Harness, digit: char) {
    harness.send(Input::chord(
        &[ShortcutKey::Super],
        ShortcutKey::Char(digit),
    ));
    harness.advance(ms(50));
}

fn text(harness: &Harness, selector: &str) -> String {
    harness.text_of(selector).unwrap_or_default()
}

fn frame_style(harness: &Harness) -> String {
    harness.attr(".ds", "style").unwrap_or_default()
}

#[test]
fn command_two_switches_and_the_frame_takes_that_spaces_look() {
    let mut harness = harness();
    assert_eq!(text(&harness, ".current"), "0");
    let first = frame_style(&harness);
    command(&mut harness, '2');
    assert_eq!(
        text(&harness, ".current"),
        "1",
        "command 2 is the second Space"
    );
    let second = frame_style(&harness);
    assert_ne!(first, second, "the root's frame variables follow the Space");
    let look = ds::style::space::presets::default_look(
        1,
        ds::style::space::look::Grain(0),
        ds::style::space::look::CardAccent::default(),
    );
    let solid = |scheme| ds::style::space::frame_vars::FrameVars::of(&look, scheme).solid;
    let (light, dark) = (solid(Scheme::Light), solid(Scheme::Dark));
    assert!(
        second.contains(&light) || second.contains(&dark),
        "{second} has neither {light} nor {dark}"
    );
    command(&mut harness, '3');
    assert_eq!(text(&harness, ".current"), "2");
    command(&mut harness, '9');
    assert_eq!(
        text(&harness, ".current"),
        "2",
        "a ninth Space that does not exist is ignored"
    );
}

#[test]
fn the_slide_follows_the_direction_and_each_space_gets_its_place_back() {
    let mut harness = harness();
    assert_eq!(text(&harness, ".slide"), "");
    let go = harness.centre(".go").expect("the button");
    harness.send(Input::click(go));
    harness.advance(ms(50));
    assert_eq!(text(&harness, ".place"), "sent");
    focus_app(&mut harness);
    command(&mut harness, '2');
    assert_eq!(
        text(&harness, ".slide"),
        "in-r",
        "a later Space comes in from the right"
    );
    assert_eq!(
        text(&harness, ".place"),
        "",
        "a Space never visited opens on the default place"
    );
    command(&mut harness, '1');
    assert_eq!(text(&harness, ".slide"), "in-l");
    assert_eq!(
        text(&harness, ".place"),
        "sent",
        "Space 1 is shown where it was left"
    );
}

#[test]
fn each_switch_is_written_once_and_the_dots_follow() {
    let mut harness = harness();
    command(&mut harness, '2');
    assert_eq!(text(&harness, ".writes"), "1");
    command(&mut harness, '2');
    assert_eq!(
        text(&harness, ".writes"),
        "1",
        "the current Space again changes nothing"
    );
}

#[test]
fn a_dot_click_switches_but_not_while_a_part_is_open() {
    let mut harness = harness();
    let third = harness
        .centre(".ds-spaces-dot-hold:nth-child(3) .ds-space-dot")
        .expect("third dot");
    harness.send(Input::click(third));
    harness.advance(ms(50));
    assert_eq!(text(&harness, ".current"), "2");
    // Right-click the first dot and choose Rename: a part is open for the first Space.
    let first = harness
        .centre(".ds-spaces-dot-hold:nth-child(1) .ds-space-dot")
        .expect("first dot");
    harness.send(Input::press(
        first,
        ds::base::press::PointerButton::Secondary,
    ));
    harness.advance(ms(100));
    let rename = harness
        .centre(".ds-menu-item:nth-child(1) .ds-menu-label")
        .expect("the menu's first row");
    harness.send(Input::click(rename));
    harness.advance(ms(400));
    assert_eq!(
        harness.count(".ds-space-rename"),
        1,
        "the name field is open"
    );
    let second = harness
        .centre(".ds-spaces-dot-hold:nth-child(2) .ds-space-dot")
        .expect("second dot");
    harness.send(Input::click(second));
    harness.advance(ms(50));
    assert_eq!(
        text(&harness, ".current"),
        "2",
        "the click switched nothing"
    );
}
