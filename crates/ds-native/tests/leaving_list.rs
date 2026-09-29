//! LeavingList on a real Blitz document, on the virtual clock: rows of different
//! heights that the caller stops listing fold out and stay drawn until exactly their batch's
//! settle; a Clear folds its rows staggered and drops them together; the rows below then heal
//! by the heights the dropped rows above each one measured, starting where they stood and
//! ending exactly in their new places; an arrival enters with `row-in` and comes to rest; a row
//! listed again while it leaves stays; and under Reduced motion a whole Clear settles at
//! Reduced's length.

use dioxus::prelude::*;
use ds::{
    Anim, Appearance, Ds, LeavingItem, LeavingList, Material, Motion, MotionLevel, StaggerIndex,
    settle,
};
use ds_native::{Clock, Harness, HarnessConfig, Viewport};
use std::cell::{Cell, RefCell};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 360,
    height: 480,
    scale_percent: 100,
};

/// Each row's height: no two alike, so a heal by the wrong row's height shows.
const HEIGHTS: [(u32, f32); 5] = [(1, 40.0), (2, 70.0), (3, 55.0), (4, 90.0), (5, 48.0)];

static SHOWN: GlobalSignal<Vec<u32>> = Signal::global(|| FIRST.with(RefCell::take));
static SETTLED: GlobalSignal<Vec<u32>> = Signal::global(Vec::new);
thread_local! {
    /// The motion setting the next harness on this thread is built with.
    static MOTION: Cell<Motion> = const { Cell::new(Motion::Standard) };
    /// The keys the next harness on this thread lists on its first render.
    static FIRST: RefCell<Vec<u32>> = const { RefCell::new(Vec::new()) };
}

fn height(n: u32) -> f32 {
    HEIGHTS
        .iter()
        .find(|(key, _)| *key == n)
        .map_or(40.0, |(_, h)| *h)
}

#[allow(non_snake_case)]
fn Column() -> Element {
    let items: Vec<LeavingItem<u32>> = SHOWN()
        .into_iter()
        .map(|n| LeavingItem {
            key: n,
            row: rsx! {
                div { id: "r{n}", style: "height:{height(n)}px", "Row {n}" }
            },
        })
        .collect();
    let settled = SETTLED()
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",");
    rsx! {
        Ds { appearance: Appearance { motion: MOTION.with(Cell::get), ..Appearance::default() }, material: Material::Window,
            LeavingList::<u32> { label: "Notifications", items, on_settled: move |key| SETTLED.write().push(key) }
            p { class: "settled", "{settled}" }
        }
    }
}

/// The row at `position` (from 1) among those drawn, leaving ones included.
fn nth(position: usize) -> String {
    format!(".ds-leaving-row:nth-child({position})")
}

fn presence(harness: &Harness, position: usize) -> Option<String> {
    harness.attr(&nth(position), "data-presence")
}

fn top(harness: &Harness, n: u32) -> f32 {
    harness
        .rect(&format!("#r{n}"))
        .unwrap_or_else(|| panic!("row {n}"))
        .origin
        .y
        .0
}

/// A heal distance, read off the row's `--dy`.
fn dy(harness: &Harness, position: usize) -> Option<f32> {
    harness.attr(&nth(position), "style").and_then(|style| {
        style
            .split(';')
            .find_map(|part| part.strip_prefix("--dy:"))
            .and_then(|value| value.trim_end_matches("px").parse().ok())
    })
}

fn show(harness: &mut Harness, keys: &[u32]) {
    harness.within(|| *SHOWN.write() = keys.to_vec());
    harness.advance(Duration::from_millis(1));
}

/// A list showing `keys` at rest, under `motion`, on the virtual clock.
fn start(keys: &[u32], motion: Motion) -> Harness {
    MOTION.with(|cell| cell.set(motion));
    FIRST.with(|first| first.replace(keys.to_vec()));
    let config = HarnessConfig::new(VIEW).with_clock(Clock::Virtual);
    let mut harness = Harness::with_config(Column, config);
    // Rows listed on the first render are simply there; their heights are measured.
    harness.advance(Duration::from_millis(50));
    harness
}

fn fold(level: MotionLevel, index: usize) -> Duration {
    settle(Anim::Fold, level, StaggerIndex::new(index))
}

fn heal(level: MotionLevel, index: usize) -> Duration {
    settle(Anim::Heal, level, StaggerIndex::new(index))
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

#[test]
fn rows_listed_first_are_there_at_once_and_measure_their_heights() {
    let harness = start(&[1, 2, 3, 4], Motion::Standard);
    assert_eq!(harness.count(".ds-leaving-row"), 4);
    for position in 1..=4 {
        assert_eq!(presence(&harness, position).as_deref(), Some("present"));
    }
    assert_eq!(top(&harness, 2) - top(&harness, 1), height(1));
    assert_eq!(top(&harness, 4) - top(&harness, 3), height(3));
}

#[test]
fn one_row_leaves_and_the_rows_below_heal_by_its_height() {
    let mut harness = start(&[1, 2, 3, 4], Motion::Standard);
    let (before3, before4) = (top(&harness, 3), top(&harness, 4));
    show(&mut harness, &[1, 3, 4]);
    let removed = ms(1);
    assert_eq!(presence(&harness, 2).as_deref(), Some("leaving"));
    assert_eq!(harness.attr(&nth(2), "data-exit").as_deref(), Some("fold"));
    let out = fold(MotionLevel::Standard, 0);
    harness.advance(out - removed - ms(1));
    assert_eq!(
        harness.count(".ds-leaving-row"),
        4,
        "drawn until it settles"
    );
    harness.advance(ms(1));
    assert_eq!(harness.count(".ds-leaving-row"), 3, "dropped at its settle");
    assert_eq!(
        presence(&harness, 1).as_deref(),
        Some("present"),
        "the row above stays"
    );
    for position in [2, 3] {
        assert_eq!(presence(&harness, position).as_deref(), Some("healing"));
        assert_eq!(
            dy(&harness, position),
            Some(height(2)),
            "heals by row 2's height"
        );
    }
    // It starts where it stood: its new place plus the heal distance.
    assert_eq!(top(&harness, 3) + height(2), before3);
    harness.advance(heal(MotionLevel::Standard, 1));
    assert_eq!(presence(&harness, 3).as_deref(), Some("present"));
    assert_eq!(top(&harness, 3), before3 - height(2));
    assert_eq!(top(&harness, 4), before4 - height(2));
    assert_eq!(top(&harness, 3), top(&harness, 1) + height(1));
    assert_eq!(harness.text_of(".settled").as_deref(), Some("2"));
}

#[test]
fn a_clear_folds_its_rows_staggered_and_heals_by_their_summed_heights() {
    let mut harness = start(&[1, 2, 3, 4, 5], Motion::Standard);
    let (first, before4, before5) = (top(&harness, 1), top(&harness, 4), top(&harness, 5));
    // Rows 1 and 3 go in one render: one batch, staggered in list order.
    show(&mut harness, &[2, 4, 5]);
    assert_eq!(presence(&harness, 1).as_deref(), Some("leaving"));
    assert_eq!(presence(&harness, 3).as_deref(), Some("leaving"));
    assert_eq!(harness.attr(&nth(1), "style").as_deref(), Some("--i:0"));
    assert_eq!(harness.attr(&nth(3), "style").as_deref(), Some("--i:1"));
    // The batch waits for its last row: the first has folded, and still nothing moves.
    let batch = fold(MotionLevel::Standard, 1);
    assert!(batch > fold(MotionLevel::Standard, 0));
    harness.advance(batch - ms(2));
    assert_eq!(harness.count(".ds-leaving-row"), 5);
    assert_eq!(presence(&harness, 2).as_deref(), Some("present"));
    harness.advance(ms(1));
    assert_eq!(harness.count(".ds-leaving-row"), 3, "dropped together");
    assert_eq!(dy(&harness, 1), Some(height(1)), "row 2 heals by row 1");
    let both = height(1) + height(3);
    assert_eq!(dy(&harness, 2), Some(both), "row 4 heals by rows 1 and 3");
    assert_eq!(dy(&harness, 3), Some(both));
    harness.advance(heal(MotionLevel::Standard, 2));
    for position in 1..=3 {
        assert_eq!(presence(&harness, position).as_deref(), Some("present"));
    }
    assert_eq!(top(&harness, 2), first);
    assert_eq!(top(&harness, 4), before4 - both);
    assert_eq!(top(&harness, 5), before5 - both);
    let mut settled: Vec<String> = harness
        .text_of(".settled")
        .unwrap_or_default()
        .split(',')
        .map(str::to_owned)
        .collect();
    settled.sort();
    assert_eq!(settled, ["1", "3"]);
}

#[test]
fn clearing_everything_settles_at_the_last_rows_stagger() {
    let mut harness = start(&[1, 2, 3, 4], Motion::Standard);
    show(&mut harness, &[]);
    assert_eq!(harness.attr(&nth(4), "style").as_deref(), Some("--i:3"));
    let batch = fold(MotionLevel::Standard, 3);
    harness.advance(batch - ms(2));
    assert_eq!(harness.count(".ds-leaving-row"), 4);
    harness.advance(ms(1));
    assert_eq!(harness.count(".ds-leaving-row"), 0);
    assert_eq!(
        harness.text_of(".settled").map(|s| s.split(',').count()),
        Some(4)
    );
}

#[test]
fn an_arrival_enters_and_comes_to_rest() {
    let mut harness = start(&[1, 2], Motion::Standard);
    show(&mut harness, &[5, 1, 2]);
    assert_eq!(presence(&harness, 1).as_deref(), Some("entering"));
    assert_eq!(
        harness.attr(".ds-leaving-list", "data-presence").as_deref(),
        Some("present"),
        "an arrival into a list at rest plays row-in"
    );
    let enter = settle(Anim::RowIn, MotionLevel::Standard, StaggerIndex::new(0));
    harness.advance(enter - ms(2));
    assert_eq!(presence(&harness, 1).as_deref(), Some("entering"));
    harness.advance(ms(1));
    assert_eq!(presence(&harness, 1).as_deref(), Some("present"));
    assert_eq!(top(&harness, 1) - top(&harness, 5), height(5));
}

#[test]
fn a_row_listed_again_while_it_leaves_stays() {
    let mut harness = start(&[1, 2, 3], Motion::Standard);
    show(&mut harness, &[1, 3]);
    assert_eq!(presence(&harness, 2).as_deref(), Some("leaving"));
    show(&mut harness, &[1, 2, 3]);
    assert_eq!(presence(&harness, 2).as_deref(), Some("present"));
    harness.advance(fold(MotionLevel::Standard, 0) * 2);
    assert_eq!(harness.count(".ds-leaving-row"), 3);
    assert_eq!(
        presence(&harness, 3).as_deref(),
        Some("present"),
        "nothing healed"
    );
    assert_eq!(harness.text_of(".settled").as_deref(), Some(""));
}

#[test]
fn under_reduced_a_clear_settles_at_reduceds_length_and_rows_snap_into_place() {
    let mut harness = start(&[1, 2, 3], Motion::Reduced);
    let before3 = top(&harness, 3);
    show(&mut harness, &[3]);
    // No stagger under Reduced: the batch is one fade's settle.
    let batch = fold(MotionLevel::Reduced, 1);
    assert_eq!(batch, fold(MotionLevel::Reduced, 0));
    harness.advance(batch - ms(2));
    assert_eq!(harness.count(".ds-leaving-row"), 3);
    harness.advance(ms(1));
    assert_eq!(harness.count(".ds-leaving-row"), 1);
    assert_eq!(dy(&harness, 1), Some(height(1) + height(2)));
    harness.advance(heal(MotionLevel::Reduced, 0));
    assert_eq!(presence(&harness, 1).as_deref(), Some("present"));
    assert_eq!(top(&harness, 3), before3 - height(1) - height(2));
}
