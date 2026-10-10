//! ShortcutField on a real Blitz document: a click makes it listen, a combination is offered, Escape
//! cancels, Backspace clears, and while it listens no key reaches the window's actions.

use chordkit::{Chord, Key as ChordKey, Modifier, Modifiers as ChordModifiers};
use dioxus::prelude::*;
use ds::keys::on_action;
use ds::prelude::*;
use ds_blitz::FocusFallback;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 420,
    height: 160,
    scale_percent: 100,
};

const WELL: &str = ".ds-shortcut-field-well";

fn start() -> Chord {
    Chord::new(ChordModifiers::of(Modifier::Super), ChordKey::Char('j'))
}

#[allow(non_snake_case)]
fn Page() -> Element {
    let keys = ds::keys::use_keys();
    let mut shortcut = use_signal(|| Some(start()));
    let mut log = use_signal(String::new);
    let mut actions = use_signal(|| 0_u32);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            div {
                onkeydown: on_action(keys, chordkit::Context::Normal, move |_| {
                    actions += 1;
                    ds::keys::ActionTaken::Yes
                }),
                ShortcutField {
                    label: "Open",
                    value: shortcut(),
                    onrecord: move |recorded: ShortcutRecorded| {
                        log.with_mut(|log| log.push_str(match recorded {
                            ShortcutRecorded::Chord(_) => "chord,",
                            ShortcutRecorded::Cleared => "cleared,",
                            ShortcutRecorded::Cancelled => "cancelled,",
                            _ => "other,",
                        }));
                        match recorded {
                            ShortcutRecorded::Chord(chord) => shortcut.set(Some(chord)),
                            ShortcutRecorded::Cleared => shortcut.set(None),
                            _ => {}
                        }
                    },
                }
                span { id: "log", "{log}" }
                span { id: "actions", "{actions}" }
            }
        }
    }
}

fn started() -> Harness {
    let config = HarnessConfig::new(VIEW)
        .with_clock(Clock::Virtual)
        .with_focus_fallback(FocusFallback::BlitzDefault);
    let mut harness = Harness::new(Page, config);
    harness.advance(Duration::from_millis(100));
    harness
}

fn listening() -> Harness {
    let mut harness = started();
    let at = harness.centre(WELL).expect("laid out"); // test-only
    harness.send(Input::click(at));
    harness.advance(Duration::from_millis(16));
    harness
}

fn text(harness: &Harness) -> String {
    harness.text_of(WELL).unwrap_or_default()
}

fn log(harness: &Harness) -> String {
    harness.text_of("#log").unwrap_or_default()
}

#[test]
fn a_click_makes_it_listen_and_it_shows_the_shortcut_otherwise() {
    let mut harness = started();
    let shown = text(&harness);
    assert!(shown.contains('J'), "the current chord is drawn: {shown:?}");
    let at = harness.centre(WELL).expect("laid out"); // test-only
    harness.send(Input::click(at));
    harness.advance(Duration::from_millis(16));
    assert_eq!(text(&harness), "Type shortcut");
}

#[test]
fn the_next_combination_is_offered_and_shown() {
    let mut harness = listening();
    harness.send(Input::chord(
        &[ShortcutKey::Super, ShortcutKey::Shift],
        ShortcutKey::Char('k'),
    ));
    assert_eq!(log(&harness), "chord,");
    let shown = text(&harness);
    assert!(shown.contains('K') && shown != "Type shortcut", "{shown:?}");
}

#[test]
fn a_bare_letter_is_not_a_shortcut_and_it_keeps_listening() {
    let mut harness = listening();
    harness.send(Input::key(ShortcutKey::Char('k')));
    assert_eq!(log(&harness), "");
    assert_eq!(text(&harness), "Type shortcut");
}

#[test]
fn escape_cancels_and_leaves_the_shortcut() {
    let mut harness = listening();
    harness.send(Input::key(ShortcutKey::Escape));
    assert_eq!(log(&harness), "cancelled,");
    assert!(text(&harness).contains('J'));
}

#[test]
fn backspace_clears_the_shortcut() {
    let mut harness = listening();
    harness.send(Input::key(ShortcutKey::Backspace));
    assert_eq!(log(&harness), "cleared,");
    assert_eq!(text(&harness), "Add shortcut");
}

#[test]
fn keys_reach_the_window_actions_only_when_it_is_not_listening() {
    let mut harness = listening();
    let before = harness.text_of("#actions").unwrap_or_default();
    harness.send(Input::chord(&[ShortcutKey::Super], ShortcutKey::Char('n')));
    assert_eq!(
        harness.text_of("#actions").unwrap_or_default(),
        before,
        "recorded, not run"
    );
    assert_eq!(log(&harness), "chord,");
    // Recording ended; the same chord now reaches the action handler above the field.
    harness.send(Input::chord(&[ShortcutKey::Super], ShortcutKey::Char('n')));
    assert_ne!(
        harness.text_of("#actions").unwrap_or_default(),
        before,
        "idle: the window hears it"
    );
}
