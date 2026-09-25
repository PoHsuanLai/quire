//! MonthGrid on a real Blitz document (design/04-COMPONENTS.md section 39; sill Q180): the
//! header's next button hands `Step::Next` to `onstep`; the new month's weeks carry the slide
//! class (`a-slide-r` for a later month, `a-slide-l` for an earlier one) right after the change
//! and drop it once the slide settles, never before `settle(Anim::SlideR)`; and a pressable day
//! hands its key to `onpick`.

#[path = "../../ds/tests/support/month_sample.rs"]
mod month_sample;

use dioxus::prelude::*;
use ds::{Anim, Appearance, DayKey, Ds, Material, MonthGrid, MotionLevel, StaggerIndex, Step};
use ds::{WeekNumbers, settle};
use ds_native::harness::settle_until;
use ds_native::{Harness, Viewport};
use month_sample::{First, SEPTEMBER, sample, shift};
use std::time::{Duration, Instant};

// `Harness::advance` lets real (wall-clock) time pass (its module documentation), so these
// tests never assert a state at one fixed instant near the slide's settle: the "still sliding"
// check comes within a millisecond of the press and is guarded to be well under half the slide;
// the settle itself is polled with `settle_until` and compared on the wall clock.

const VIEW: Viewport = Viewport {
    width: 480,
    height: 480,
    scale_percent: 100,
};

/// A month the header steps, and a log of every step and pick.
#[allow(non_snake_case)]
fn MonthApp() -> Element {
    let mut month = use_signal(|| SEPTEMBER);
    let mut log = use_signal(Vec::<String>::new);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Popover,
            MonthGrid {
                data: sample(month(), First::Monday),
                weeks: WeekNumbers::Show,
                onstep: move |step: Step| {
                    log.with_mut(|log| log.push(format!("{step:?}")));
                    month.set(shift(month(), step));
                },
                onpick: move |day: DayKey| {
                    log.with_mut(|log| log.push(format!("{}-{}-{}", day.year, day.month, day.day)));
                },
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn press(harness: &mut Harness, selector: &str) {
    let at = harness
        .centre(selector)
        .unwrap_or_else(|| panic!("{selector} is missing:\n{}", harness.html()));
    harness.click(at);
}

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

/// How long the month's slide takes to settle at the Standard level.
fn slide() -> Duration {
    settle(Anim::SlideR, MotionLevel::Standard, StaggerIndex::default())
}

fn sliding(harness: &Harness, class: &str) -> bool {
    harness.has_class(".ds-month-weeks", class)
}

/// Press a step button and watch its slide: present at once, gone only after a full slide.
fn step_and_watch(harness: &mut Harness, label: &str, class: &str) {
    let pressed = Instant::now();
    press(harness, &format!(".ds-icon-button[*|aria-label='{label}']"));
    harness.advance(ms(1));
    let looked = Instant::now();
    assert!(
        looked.duration_since(pressed) < slide() / 2,
        "the first look came well inside the slide"
    );
    assert!(
        sliding(harness, class),
        "{class} right after the change:\n{}",
        harness.html()
    );
    assert_eq!(
        harness.attr(".ds-month-weeks", "data-pulse").as_deref(),
        Some("a")
    );
    let rested = settle_until(harness, |harness| !sliding(harness, class));
    assert!(
        rested.duration_since(pressed) >= slide(),
        "the class stays for the whole slide: {:?}",
        rested.duration_since(pressed)
    );
    assert_eq!(harness.attr(".ds-month-weeks", "data-pulse"), None);
}

#[test]
fn a_step_slides_the_new_month_in_once() {
    let mut harness = Harness::new(MonthApp, VIEW);
    harness.advance(ms(50));
    assert!(!harness.has_class(".ds-month-weeks", "a-slide-r"));
    assert!(!harness.has_class(".ds-month-weeks", "a-slide-l"));
    assert_eq!(
        harness.text_of(".ds-month-title").as_deref(),
        Some("September 2026")
    );

    step_and_watch(&mut harness, "Next month", "a-slide-r");
    assert_eq!(log(&harness), "Next");
    assert_eq!(
        harness.text_of(".ds-month-title").as_deref(),
        Some("October 2026")
    );
    assert!(
        !sliding(&harness, "a-slide-l"),
        "a later month never slides from the left"
    );

    step_and_watch(&mut harness, "Previous month", "a-slide-l");
    assert_eq!(log(&harness), "Next,Previous");
    assert_eq!(
        harness.text_of(".ds-month-title").as_deref(),
        Some("September 2026")
    );
}

#[test]
fn a_pressed_day_hands_over_its_key() {
    let mut harness = Harness::new(MonthApp, VIEW);
    harness.advance(ms(50));
    assert_eq!(log(&harness), "");
    press(&mut harness, ".ds-month-day[*|aria-current=date]");
    harness.advance(ms(20));
    assert_eq!(log(&harness), "2026-9-26");
    assert!(
        !sliding(&harness, "a-slide-r") && !sliding(&harness, "a-slide-l"),
        "a render that keeps the month slides nothing"
    );
    press(&mut harness, ".ds-month-day[*|data-place=after]");
    harness.advance(ms(20));
    assert_eq!(
        log(&harness),
        "2026-9-26,2026-10-1",
        "a neighbour's day is its own date"
    );
}
