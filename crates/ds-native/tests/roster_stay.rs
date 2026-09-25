//! An exit taken back (mailo gaps 3, item 1): a row whose fold is playing is restored in place
//! by `Roster::stay` before the fold settles. The row is present again with no exit, and the
//! rows below never heal, even after the fold's settle time has long passed.

use dioxus::prelude::*;
use ds::{
    Anim, AnimatedList, Appearance, Button, ButtonVariant, Ds, Emphasis, Exit, ListPresence,
    ListRow, Material, Point, PulseKey, Px, RowPitch, Selection, StaggerIndex, Stayed, settle,
    use_roster,
};
use ds_native::harness::settle_until;
use ds_native::{Harness, Viewport};
use std::time::{Duration, Instant};

const VIEW: Viewport = Viewport {
    width: 480,
    height: 360,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

#[allow(non_snake_case)]
fn StayApp() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, StayList {} }
    }
}

/// Three rows; clicking one folds it away, Undo takes the last fold back.
#[allow(non_snake_case)]
fn StayList() -> Element {
    let mut keys = use_signal(|| vec![1u32, 2, 3]);
    let mut last = use_signal(|| None::<u32>);
    let roster = use_roster(keys(), RowPitch(Px(79.0)));
    rsx! {
        Button {
            variant: ButtonVariant::Secondary,
            label: "Undo",
            onclick: move |_| {
                if let Some(key) = last() {
                    assert_eq!(roster.stay(key), Ok(Stayed::Restored));
                    keys.set(vec![1, 2, 3]);
                    last.set(None);
                }
            },
        }
        AnimatedList { label: "Threads", presence: ListPresence::Present,
            for entry in roster.entries() {
                ListRow {
                    key: "{entry.key}",
                    selection: Selection::Unselected,
                    emphasis: Emphasis::Plain,
                    index: entry.index,
                    presence: entry.presence,
                    name: format!("Sender {}", entry.key),
                    via: None,
                    subject: format!("Subject {}", entry.key),
                    snippet: None,
                    time: "09:41",
                    tags: rsx! {},
                    star: None,
                    star_pulse: PulseKey::rest(Anim::Bump),
                    strip: None,
                    onclick: move |_| {
                        roster.leave(entry.key, Exit::Fold, Emphasis::Plain);
                        keys.retain(|key| *key != entry.key);
                        last.set(Some(entry.key));
                    },
                }
            }
        }
    }
}

fn centre(harness: &Harness, selector: &str) -> Point {
    harness
        .centre(selector)
        .unwrap_or_else(|| panic!("{selector} is not in the document:\n{}", harness.html()))
}

fn row(n: usize) -> String {
    format!(".ds-row:nth-child({n})")
}

#[test]
fn an_exit_stayed_before_it_settles_restores_the_row_and_heals_nothing() {
    let mut harness = Harness::new(StayApp, VIEW);
    harness.advance(ms(1500));
    assert_eq!(harness.count(".ds-row"), 3);
    let tops: Vec<_> = (1..=3)
        .map(|n| harness.rect(&row(n)).map(|rect| rect.origin.y))
        .collect();

    harness.click(centre(&harness, &row(1)));
    assert_eq!(
        harness.attr(&row(1), "data-presence").as_deref(),
        Some("leaving")
    );
    assert_eq!(harness.attr(&row(1), "data-exit").as_deref(), Some("fold"));

    // Undo well inside the fold: 200 ms of `settle(Fold)`'s 454 ms at Standard.
    let fold = settle(
        Anim::Fold,
        ds::MotionLevel::Standard,
        StaggerIndex::default(),
    );
    harness.advance(ms(200));
    harness.click(centre(&harness, ".ds-button"));
    assert_eq!(
        harness.attr(&row(1), "data-presence").as_deref(),
        Some("present")
    );
    assert_eq!(harness.attr(&row(1), "data-exit"), None, "the exit is gone");

    // Past where the fold would have settled, and past any heal it would have started.
    harness.advance(fold + ms(600));
    assert_eq!(harness.count(".ds-row"), 3, "{}", harness.html());
    for n in 1..=3 {
        assert_eq!(
            harness.attr(&row(n), "data-presence").as_deref(),
            Some("present"),
            "row {n}"
        );
        let style = harness.attr(&row(n), "style").unwrap_or_default();
        assert!(!style.contains("--dy"), "row {n} heals: {style}");
    }
    let after: Vec<_> = (1..=3)
        .map(|n| harness.rect(&row(n)).map(|rect| rect.origin.y))
        .collect();
    assert_eq!(after, tops, "every row is where it was");
}

#[test]
fn a_row_folded_again_after_a_stay_settles_on_its_own_clock() {
    let mut harness = Harness::new(StayApp, VIEW);
    harness.advance(ms(1500));
    harness.click(centre(&harness, &row(2)));
    harness.advance(ms(200));
    harness.click(centre(&harness, ".ds-button"));
    // Fold it again 150 ms later: the first fold's timer, had it survived the stay, would drop
    // the row about 100 ms into the second fold.
    harness.advance(ms(150));
    let refolded = Instant::now();
    harness.click(centre(&harness, &row(2)));
    // Well under half the second fold's own settle(Fold) (~454 ms), and comfortably past the
    // ~104 ms mark where the first fold's stale timer would have dropped the row had it
    // survived (fixed 2026-09-25, FINDINGS "Timing tests"): the old 250 ms check was 55 % of
    // the window, over the margin a loaded machine's overshoot on `advance` can eat into.
    harness.advance(ms(200));
    assert_eq!(
        harness.count(".ds-row"),
        3,
        "the second fold is still playing, and the stale first-fold timer never fired"
    );
    assert_eq!(
        harness.attr(&row(2), "data-presence").as_deref(),
        Some("leaving")
    );
    let fold = settle(
        Anim::Fold,
        ds::MotionLevel::Standard,
        StaggerIndex::default(),
    );
    let dropped = settle_until(&mut harness, |h| h.count(".ds-row") == 2);
    assert!(
        dropped.duration_since(refolded) >= fold,
        "the second fold dropped the row only once its own full settle had run: {:?}",
        dropped.duration_since(refolded)
    );
}
