//! A `Scrubber` on a real Blitz document (design/30 section 2.1a): hovering shows the time under
//! the pointer and leaving hides it, a press and a drag report places along the bar and keep
//! following the pointer outside it (the press captures it), the release ends the scrub, a click
//! is a start and an end at one place, Escape abandons a drag, and the keys ask for places.

use dioxus::prelude::*;
use ds::components::controls::scrubber::Scrubber;
use ds::components::controls::scrubber_model::BufferedRange;
use ds::motion::spring::Millis;
use ds::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 400,
    height: 120,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// A bar for a recording 200 s long, 200 px wide, so a pixel is five seconds; `.log` lists what
/// the handlers heard.
#[allow(non_snake_case)]
fn Page() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut note = move |text: String| log.with_mut(|log| log.push(text));
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "margin:40px 20px; width:200px",
                Scrubber {
                    label: "Position",
                    position: Fraction(250),
                    length: Millis(200_000),
                    buffered: vec![BufferedRange { from: Fraction(0), to: Fraction(600) }],
                    onscrubstart: move |at: Fraction| note(format!("start {}", at.0)),
                    onscrub: move |at: Fraction| note(format!("scrub {}", at.0)),
                    onscrubend: move |at: Fraction| note(format!("end {}", at.0)),
                    onscrubcancel: move |()| note("cancel".to_owned()),
                    onseek: move |at: Fraction| note(format!("seek {}", at.0)),
                }
            }
            p { class: "log", {log().join("; ")} }
        }
    }
}

fn harness() -> Harness {
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(100));
    harness
}

fn at(harness: &Harness, share: f32) -> Point {
    let track = harness.rect(".ds-scrubber-track").expect("the track");
    Point {
        x: Px(track.origin.x.0 + track.size.width.0 * share),
        y: Px(track.origin.y.0 + track.size.height.0 / 2.0),
    }
}

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

#[test]
fn the_tooltip_reads_the_time_under_the_pointer_and_goes_with_it() {
    let mut harness = harness();
    assert_eq!(harness.count(".ds-scrubber-tooltip"), 0, "none at rest");
    assert_eq!(
        harness.count(".ds-scrubber-buffered"),
        1,
        "a band for the loaded stretch"
    );
    harness.send(Input::pointer_move(at(&harness, 0.5)));
    harness.advance(ms(100));
    harness.send(Input::pointer_move(at(&harness, 0.5)));
    harness.advance(ms(100));
    assert_eq!(
        harness.attr(".ds-scrubber", "data-state").as_deref(),
        Some("hover")
    );
    assert_eq!(
        harness.text_of(".ds-scrubber-tooltip").as_deref(),
        Some("1:40"),
        "the middle of 200 s"
    );
    harness.send(Input::pointer_move(Point {
        x: Px(380.0),
        y: Px(110.0),
    }));
    harness.advance(ms(100));
    assert_eq!(
        harness.count(".ds-scrubber-tooltip"),
        0,
        "gone with the pointer"
    );
}

#[test]
fn a_drag_reports_places_and_follows_the_pointer_outside_the_bar() {
    let mut harness = harness();
    harness.send(Input::pointer_move(at(&harness, 0.25)));
    harness.advance(ms(100));
    harness.send(Input::pointer_down(at(&harness, 0.25)));
    harness.advance(ms(100));
    assert_eq!(
        harness.attr(".ds-scrubber", "data-state").as_deref(),
        Some("dragging")
    );
    harness.send(Input::pointer_move(at(&harness, 0.75)));
    harness.advance(ms(50));
    // Past the bar's right end and below it: the capture still reads the place, held at the end.
    let outside = Point {
        x: Px(390.0),
        y: Px(115.0),
    };
    harness.send(Input::pointer_move(outside));
    harness.advance(ms(50));
    harness.send(Input::pointer_up(outside));
    harness.advance(ms(100));
    assert_eq!(log(&harness), "start 250; scrub 750; scrub 1000; end 1000");
}

#[test]
fn a_click_is_a_start_and_an_end_at_one_place() {
    let mut harness = harness();
    harness.send(Input::pointer_move(at(&harness, 0.4)));
    harness.advance(ms(100));
    harness.send(Input::pointer_down(at(&harness, 0.4)));
    harness.advance(ms(100));
    harness.send(Input::pointer_up(at(&harness, 0.4)));
    harness.advance(ms(100));
    assert_eq!(log(&harness), "start 400; end 400");
}

#[test]
fn escape_abandons_a_drag_and_the_keys_ask_for_places() {
    let mut harness = harness();
    harness.send(Input::pointer_move(at(&harness, 0.25)));
    harness.advance(ms(100));
    harness.send(Input::pointer_down(at(&harness, 0.25)));
    harness.advance(ms(100));
    harness.send(Input::key(ShortcutKey::Escape));
    harness.advance(ms(50));
    assert_eq!(log(&harness), "start 250; cancel");
    harness.send(Input::key(ShortcutKey::Right));
    harness.send(Input::key(ShortcutKey::Left));
    harness.send(Input::key(ShortcutKey::End));
    harness.send(Input::key(ShortcutKey::Home));
    assert_eq!(
        log(&harness),
        "start 250; cancel; seek 270; seek 230; seek 1000; seek 0",
        "a step is a fiftieth of the length from where the caller says it is"
    );
}
