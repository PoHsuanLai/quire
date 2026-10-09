//! `RawKeySurface` on a real Blitz document: every key down, repeat and up reaches the app with
//! its physical code, the surface keeps Tab, and the harness's raw keys, Super and paste chord
//! are what the window sends. Also the host measuring an element it found by selector.

use dioxus::prelude::*;
use ds::edit::raw_key::{KeyPhase, RawKey};
use ds::focus::select::Select;
use ds::focus::soon::focus_soon_told;
use ds::host::measure::Measured;
use ds::prelude::*;
use ds_harness::{
    Clock, Driver, Harness, HarnessConfig, Input, PasteChord, Query, RawKeyInput, RawKeyPhase,
    Viewport,
};
use keyboard_types::{Code, Key, Location, Modifiers};
use std::cell::RefCell;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 480,
    height: 320,
    scale_percent: 100,
};

thread_local! {
    static HEARD: RefCell<Vec<RawKey>> = const { RefCell::new(Vec::new()) };
}

fn heard() -> Vec<RawKey> {
    HEARD.with(|heard| heard.borrow().clone())
}

#[allow(non_snake_case)]
fn Page() -> Element {
    let mut told = use_signal(|| 0u8);
    let mut downs = use_signal(|| 0u8);
    let mut element = use_signal(|| None::<std::rc::Rc<MountedData>>);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            RawKeySurface {
                style: "display:block;width:200px;height:80px",
                onpointerdown: move |_: PointerEvent| downs += 1,
                on_key: move |key: RawKey| HEARD.with(|heard| heard.borrow_mut().push(key)),
                common: Common {
                    mounted: Some(EventHandler::new(move |event: MountedEvent| element.set(Some(event.data())))),
                    ..Common::default()
                },
                div { class: "grid", style: "width:120px;height:60px", "grid" }
            }
            button {
                class: "focus",
                onclick: move |_| {
                    if let Some(el) = element() {
                        focus_soon_told(el, Select::None, EventHandler::new(move |()| told += 1));
                    }
                },
                "focus"
            }
            p { class: "told", "{told}" }
            p { class: "downs", "{downs}" }
        }
    }
}

fn start() -> Harness {
    HEARD.with(|heard| heard.borrow_mut().clear());
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(300));
    let at = harness.centre(".focus").expect("button");
    harness.send(Input::click(at));
    harness.advance(Duration::from_millis(300));
    harness
}

fn semicolon(phase: RawKeyPhase) -> Input {
    Input::raw_key(RawKeyInput::down(Key::Character(";".into()), Code::Semicolon).in_phase(phase))
}

#[test]
fn focus_soon_told_tells_once_the_surface_has_the_keyboard() {
    let harness = start();
    assert_eq!(harness.text_of(".told").as_deref(), Some("1"));
}

#[test]
fn a_key_arrives_down_repeating_and_up_with_its_physical_code() {
    let mut harness = start();
    for phase in [RawKeyPhase::Down, RawKeyPhase::Repeat, RawKeyPhase::Up] {
        harness.send(semicolon(phase));
    }
    let got: Vec<_> = heard()
        .into_iter()
        .map(|key| (key.phase, key.code, key.text))
        .collect();
    let text = Some(";".to_owned());
    assert_eq!(
        got,
        vec![
            (KeyPhase::Press, Code::Semicolon, text.clone()),
            (KeyPhase::Repeat, Code::Semicolon, text),
            (KeyPhase::Release, Code::Semicolon, None),
        ]
    );
}

#[test]
fn a_press_with_a_platform_text_reports_it_and_a_chord_types_nothing_of_its_own() {
    let mut harness = start();
    harness.send(Input::raw_key(
        RawKeyInput::down(Key::Character("e".into()), Code::KeyE).typing("é"),
    ));
    harness.send(Input::raw_key(
        RawKeyInput::down(Key::Character("c".into()), Code::KeyC).with_mods(Modifiers::CONTROL),
    ));
    let got: Vec<_> = heard().into_iter().map(|key| key.text).collect();
    // The event Dioxus hands the app carries no platform text yet, so "é" degrades to the
    // logical character; the chord has none either way.
    assert_eq!(got, vec![Some("e".to_owned()), None]);
}

#[test]
fn tab_stays_in_the_surface_and_the_side_of_a_key_is_kept() {
    let mut harness = start();
    harness.send(Input::raw_key(
        RawKeyInput::down(Key::Shift, Code::ShiftRight).at_location(Location::Right),
    ));
    harness.send(Input::raw_key(RawKeyInput::down(Key::Tab, Code::Tab)));
    let got = heard();
    assert_eq!(got.len(), 2);
    assert_eq!(got[0].location, Location::Right);
    assert_eq!(got[1].key, Key::Tab);
}

#[test]
fn super_chords_carry_super_and_not_meta() {
    let mut harness = start();
    harness.send(Input::chord(
        &[ds::prelude::ShortcutKey::Super],
        ds::prelude::ShortcutKey::Char('t'),
    ));
    let key = heard().into_iter().last().expect("a key");
    assert_eq!(key.modifiers, Modifiers::SUPER);
}

#[test]
fn a_paste_chord_you_choose_is_the_one_pressed() {
    let mut harness = start();
    harness.send(Input::paste_with(PasteChord::CtrlShiftV, "<b>x</b>", "x"));
    let downs: Vec<_> = heard()
        .into_iter()
        .filter(|key| key.phase == KeyPhase::Press)
        .map(|key| (key.code, key.modifiers))
        .collect();
    assert_eq!(
        downs,
        vec![(Code::KeyV, Modifiers::CONTROL | Modifiers::SHIFT)]
    );
}

#[test]
fn an_element_found_by_selector_can_be_measured() {
    let harness = start();
    let Measured::At(rect) = harness.measure_found(".grid") else {
        panic!("the found element was not measured");
    };
    assert_eq!((rect.size.width.0, rect.size.height.0), (120.0, 60.0));
    assert!(matches!(
        harness.measure_found(".nothing-here"),
        Measured::Unknown
    ));
}

#[test]
fn the_surface_passes_a_pointer_press_through_and_takes_the_style_it_is_given() {
    let mut harness = start();
    let at = harness.centre(".grid").expect("grid");
    harness.send(Input::click(at));
    assert_eq!(harness.text_of(".downs").as_deref(), Some("1"));
    assert!(harness.html().contains("width:200px"), "{}", harness.html());
}
