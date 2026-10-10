//! ShortcutField with `accepts: SingleKeys` on a real Blitz document: a bare key and Delete are
//! recorded, Escape and Backspace still cancel and clear, and `more` draws a cap per extra key
//! beside the value without being touched by a recording.

use chordkit::{Chord, Key as ChordKey, Modifiers as ChordModifiers};
use dioxus::prelude::*;
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

fn plain(c: char) -> Chord {
    Chord::new(ChordModifiers::default(), ChordKey::Char(c))
}

#[allow(non_snake_case)]
fn Page() -> Element {
    let mut shortcut = use_signal(|| Some(plain('j')));
    let mut log = use_signal(String::new);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            ShortcutField {
                label: "Next",
                value: shortcut(),
                accepts: ShortcutKinds::SingleKeys,
                more: vec![plain('n')],
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
        }
    }
}

fn listening() -> Harness {
    let config = HarnessConfig::new(VIEW)
        .with_clock(Clock::Virtual)
        .with_focus_fallback(FocusFallback::BlitzDefault);
    let mut harness = Harness::new(Page, config);
    harness.advance(Duration::from_millis(100));
    let at = harness.centre(WELL).expect("laid out"); // test-only
    harness.send(Input::click(at));
    harness.advance(Duration::from_millis(16));
    harness
}

fn log(harness: &Harness) -> String {
    harness.text_of("#log").unwrap_or_default()
}

#[test]
fn the_value_and_each_extra_key_are_caps_in_the_well() {
    let mut idle = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    idle.advance(Duration::from_millis(100));
    assert_eq!(
        idle.count(".ds-shortcut-field-caps .ds-key-equivalent-key"),
        2,
        "the value and one extra key"
    );
}

#[test]
fn a_bare_key_is_recorded_and_the_extra_key_stays() {
    let mut harness = listening();
    harness.send(Input::key(ShortcutKey::Char('k')));
    assert_eq!(log(&harness), "chord,");
    let shown = harness.text_of(WELL).unwrap_or_default();
    assert!(shown.contains('K') && shown.contains('N'), "{shown:?}");
}

#[test]
fn delete_is_recorded_as_a_key_and_backspace_clears() {
    let mut harness = listening();
    harness.send(Input::key(ShortcutKey::Delete));
    assert_eq!(log(&harness), "chord,");
    let at = harness.centre(WELL).expect("laid out"); // test-only
    harness.send(Input::click(at));
    harness.advance(Duration::from_millis(16));
    harness.send(Input::key(ShortcutKey::Backspace));
    assert_eq!(log(&harness), "chord,cleared,");
}
