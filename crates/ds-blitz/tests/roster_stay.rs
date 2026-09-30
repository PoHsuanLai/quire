//! An exit taken back: a row whose fold is playing is restored in place when its key is listed
//! again before the fold settles. The row is present again with no exit, and the rows below never
//! heal, even after the fold's settle time has long passed.

use dioxus::prelude::*;
use ds::{
    Anim, Appearance, Button, Ds, Emphasis, List, ListItem, Material, Point, RowState, Selection,
    ThreadRow, settle,
};
use ds_harness::harness::settle_until;
use ds_harness::{Clock, Harness, HarnessConfig, Viewport};
use std::time::Duration;

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

/// Three rows; clicking one folds it away, Undo lists its key again.
#[allow(non_snake_case)]
fn StayList() -> Element {
    let mut keys = use_signal(|| vec![1u32, 2, 3]);
    let mut last = use_signal(|| None::<u32>);
    let items: Vec<ListItem<u32>> = keys()
        .into_iter()
        .map(|key| {
            ListItem::row(
                key,
                format!("Subject {key}"),
                rsx! {
                    ThreadRow {
                        state: RowState { selection: Selection::Unselected, emphasis: Emphasis::Plain, ..RowState::default() },
                        name: format!("Sender {key}"),
                        via: None,
                        subject: format!("Subject {key}"),
                        snippet: None,
                        time: "09:41",
                        tags: rsx! {},
                        star: None,
                        strip: None,
                        onclick: move |_| {
                            keys.retain(|shown| *shown != key);
                            last.set(Some(key));
                        },
                    }
                },
            )
        })
        .collect();
    rsx! {
        Button {
            label: "Undo",
            onclick: move |_| {
                if last().is_some() {
                    keys.set(vec![1, 2, 3]);
                    last.set(None);
                }
            },
        }
        List::<u32> { label: "Threads", items }
    }
}

fn centre(harness: &Harness, selector: &str) -> Point {
    harness
        .centre(selector)
        .unwrap_or_else(|| panic!("{selector} is not in the document:\n{}", harness.html()))
}

fn row(n: usize) -> String {
    format!(".ds-list-item:nth-child({n})")
}

#[test]
fn an_exit_stayed_before_it_settles_restores_the_row_and_heals_nothing() {
    let mut harness = Harness::new(StayApp, VIEW);
    harness.advance(ms(1500));
    assert_eq!(harness.count(".ds-list-item"), 3);
    let tops: Vec<_> = (1..=3)
        .map(|n| harness.rect(&row(n)).map(|rect| rect.origin.y))
        .collect();

    harness.click(centre(&harness, &row(1)));
    assert_eq!(
        harness.attr(&row(1), "data-presence").as_deref(),
        Some("leaving")
    );
    assert_eq!(harness.attr(&row(1), "data-exit").as_deref(), Some("row"));

    // Undo well inside the exit: 100 ms of `settle(RowOut)`.
    let fold = settle(Anim::RowOut, ds::MotionLevel::Standard);
    harness.advance(ms(100));
    harness.click(centre(&harness, ".ds-button"));
    assert_eq!(
        harness.attr(&row(1), "data-presence").as_deref(),
        Some("present")
    );
    assert_eq!(harness.attr(&row(1), "data-exit"), None, "the exit is gone");

    // Past where the fold would have settled, and past any heal it would have started.
    harness.advance(fold + ms(600));
    assert_eq!(harness.count(".ds-list-item"), 3, "{}", harness.html());
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
    let mut harness =
        Harness::with_config(StayApp, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(1500));
    harness.click(centre(&harness, &row(2)));
    harness.advance(ms(100));
    harness.click(centre(&harness, ".ds-button"));
    // Fold it again 150 ms later: the first fold's timer, had it survived the stay, would drop
    // the row about 100 ms into the second fold.
    harness.advance(ms(150));
    let refolded = harness.now();
    harness.click(centre(&harness, &row(2)));
    // Well under half the second fold's own settle(Fold) (~454 ms), and comfortably past the
    // ~104 ms mark where the first fold's stale timer would have dropped the row had it
    // survived (fixed 2026-09-25, FINDINGS "Timing tests"): the old 250 ms check was 55 % of
    // the window, over the margin a loaded machine's overshoot on `advance` can eat into.
    harness.advance(ms(100));
    assert_eq!(
        harness.count(".ds-list-item"),
        3,
        "the second fold is still playing, and the stale first-fold timer never fired"
    );
    assert_eq!(
        harness.attr(&row(2), "data-presence").as_deref(),
        Some("leaving")
    );
    let fold = settle(Anim::RowOut, ds::MotionLevel::Standard);
    let dropped = settle_until(&mut harness, |h| h.count(".ds-list-item") == 2);
    assert!(
        dropped.duration_since(refolded) >= fold,
        "the second fold dropped the row only once its own full settle had run: {:?}",
        dropped.duration_since(refolded)
    );
}
