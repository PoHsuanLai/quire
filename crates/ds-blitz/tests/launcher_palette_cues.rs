//! On a real Blitz document: the command palette replaces a result set in place, and a group's
//! Show More brings the added rows in downward while its Show Less heals what follows up by
//! their height (design/26 section 5.4's group expand). A group growing without its action is a
//! new result set: nothing moves. Reduced keeps the level's tokens. Every moment ends at 0
//! frames (R3).

use dioxus::prelude::*;
use ds::detail::{Detailed, Moment};
use ds::{
    Appearance, CommandPalette, CommandPaletteHost, Ds, Material, Motion, PaletteGroup,
    PaletteGroups, ShortcutKey,
};
use ds_harness::harness::{assert_settles_to_zero_frames, settle_until};
use ds_harness::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 640,
    height: 640,
    scale_percent: 100,
};

/// The results as the launcher sees them: sill's `ListState`, cut down.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum List {
    Waiting,
    Rows { answer: u8, expanded: Expanded },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Expanded {
    No,
    Yes,
}

impl Detailed for List {
    fn moment(from: &Self, to: &Self) -> Moment {
        match (from, to) {
            (List::Waiting, List::Waiting) => Moment::Rest,
            (List::Waiting, List::Rows { .. }) => Moment::Appear,
            (List::Rows { .. }, List::Rows { .. }) | (List::Rows { .. }, List::Waiting) => {
                Moment::Change
            }
        }
    }

    fn first(state: &Self) -> Moment {
        match state {
            List::Waiting => Moment::Rest,
            List::Rows { .. } => Moment::Appear,
        }
    }
}

static LIST: GlobalSignal<List> = Signal::global(|| List::Waiting);
static MOTION: GlobalSignal<Motion> = Signal::global(|| Motion::Standard);
/// Rows the Apps group gains with no action run (a query widening it).
static WIDER: GlobalSignal<u8> = Signal::global(|| 0);

fn item(value: u8, title: String) -> ds::PaletteRow<u8> {
    ds::PaletteRow::new(value, title)
}

fn groups(list: List, wider: u8) -> PaletteGroups<u8> {
    let List::Rows { answer, expanded } = list else {
        return PaletteGroups::default();
    };
    let (count, label, next) = match expanded {
        Expanded::No => (2, "Show More", Expanded::Yes),
        Expanded::Yes => (6, "Show Less", Expanded::No),
    };
    let apps = (0..count + wider)
        .map(|n| item(n, format!("App {answer}.{n}")))
        .collect();
    PaletteGroups(vec![
        PaletteGroup::list("Applications", apps).with_action(
            label,
            EventHandler::new(move |()| {
                *LIST.write() = List::Rows {
                    answer,
                    expanded: next,
                }
            }),
        ),
        PaletteGroup::list("Settings", vec![item(90, format!("Displays {answer}"))]),
    ])
}

#[allow(non_snake_case)]
fn Launcher() -> Element {
    let list = LIST();
    rsx! {
        Ds { appearance: Appearance { motion: MOTION(), ..Appearance::default() }, material: Material::Sheet,
            div { style: "width:600px;height:600px",
                CommandPalette::<u8> {
                    label: "Launch",
                    placeholder: "Search",
                    query: String::new(),
                    tokens: Vec::new(),
                    groups: groups(list, WIDER()),
                    empty: "Nothing",
                    oninput: move |_| {},
                    onpick: move |_| {},
                    onclose: move |()| {},
                    host: CommandPaletteHost::Surface,
                    id: "card".to_string(),
                }
            }
        }
    }
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn set(harness: &mut Harness, list: List) {
    harness.within(|| *LIST.write() = list);
    harness.advance(ms(20));
}

fn rows(answer: u8) -> List {
    List::Rows {
        answer,
        expanded: Expanded::No,
    }
}

/// Down until the header's action is selected, then Enter.
fn run_action(harness: &mut Harness) {
    for _ in 0..8 {
        if harness.count(".ds-section-header-action[*|data-selected=true]") == 1 {
            break;
        }
        harness.key(ShortcutKey::Down);
        harness.advance(ms(10));
    }
    harness.key(ShortcutKey::Enter);
    harness.advance(ms(20));
}

fn open_quietly(harness: &mut Harness) {
    set(harness, rows(1));
    assert_settles_to_zero_frames(harness);
}

#[test]
fn a_result_set_replaces_the_last_in_place() {
    let mut harness = Harness::new(Launcher, VIEW);
    harness.advance(ms(50));
    set(&mut harness, rows(1));
    assert_eq!(harness.count("[*|data-row-motion]"), 0);
    assert_settles_to_zero_frames(&mut harness);
    set(&mut harness, rows(2));
    assert_eq!(
        harness.text_of("#card .ds-row .ds-row-title").as_deref(),
        Some("App 2.0")
    );
    assert_eq!(harness.count("[*|data-row-motion]"), 0);
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn show_more_enters_the_added_rows_and_show_less_heals_by_their_height() {
    let mut harness = Harness::new(Launcher, VIEW);
    open_quietly(&mut harness);
    run_action(&mut harness);
    assert_eq!(harness.count("#card .ds-row"), 7);
    assert_eq!(
        harness.count("[*|data-row-motion=in]"),
        4,
        "the four added rows"
    );
    settle_until(&mut harness, |h| h.count("[*|data-row-motion]") == 0);
    assert_settles_to_zero_frames(&mut harness);

    run_action(&mut harness);
    assert_eq!(harness.count("#card .ds-row"), 3);
    assert_eq!(harness.count("[*|data-row-motion=heal-from]"), 1);
    assert_eq!(
        harness
            .text_of("[*|data-row-motion=heal-from] .ds-row-title")
            .as_deref(),
        Some("App 1.1"),
        "the last row kept"
    );
    let style = harness
        .attr("#card .ds-palette-list", "style")
        .unwrap_or_default();
    let dy: f32 = style
        .strip_prefix("--dy:")
        .and_then(|rest| rest.strip_suffix("px"))
        .and_then(|px| px.parse().ok())
        .unwrap_or_else(|| panic!("the list heals by --dy: {style:?}"));
    assert!(dy > 4.0 * 20.0, "four rows' height, not {dy}");
    settle_until(&mut harness, |h| h.count("[*|data-row-motion]") == 0);
    assert_eq!(harness.attr("#card .ds-palette-list", "style"), None);
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn a_click_on_show_more_plays_the_same_expand() {
    let mut harness = Harness::new(Launcher, VIEW);
    open_quietly(&mut harness);
    let at = harness
        .centre(".ds-section-header-action")
        .expect("the action");
    harness.click(at);
    harness.advance(ms(20));
    assert_eq!(harness.count("[*|data-row-motion=in]"), 4);
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn a_group_growing_without_its_action_is_a_new_result_set() {
    let mut harness = Harness::new(Launcher, VIEW);
    open_quietly(&mut harness);
    harness.within(|| *WIDER.write() = 3);
    harness.advance(ms(20));
    assert_eq!(harness.count("#card .ds-row"), 6);
    assert_eq!(harness.count("[*|data-row-motion]"), 0);
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn reduced_plays_the_expand_at_its_own_tokens_and_settles() {
    let mut harness = Harness::new(Launcher, VIEW);
    harness.within(|| *MOTION.write() = Motion::Reduced);
    set(&mut harness, rows(1));
    assert_settles_to_zero_frames(&mut harness);
    run_action(&mut harness);
    settle_until(&mut harness, |h| h.count("[*|data-row-motion]") == 0);
    assert_eq!(harness.count("#card .ds-row"), 7);
    assert_settles_to_zero_frames(&mut harness);
}
