//! Not every run of a command palette group's action goes through the palette's own
//! Enter or click — `sill debug launcher-key enter` steps the keyboard machine directly, a demo
//! may run the action from its own button — and hands the palette the next `groups` with no
//! Enter or click of its own for the palette to have seen. `ds::PaletteHandle::mark_group_action`
//! (from `use_palette_handle`) lets that caller book the change first: the palette then plays
//! that group's Show More or Show Less exactly as it would its own (design/26 section 5.6).
//! Unmarked, the same caller-driven change plays nothing, as any other new result set does.
//!
//! Run on the virtual clock for exact timing. Every moment ends at 0 frames.

use dioxus::prelude::*;
use ds::{
    Appearance, Availability, CommandPalette, CommandPaletteHost, Ds, Material, MenuEntry,
    MenuTrail, PaletteGroup, PaletteGroups, PaletteHandle, use_palette_handle,
};
use ds_native::harness::{assert_settles_to_zero_frames, settle_until};
use ds_native::{Clock, Harness, HarnessConfig, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 640,
    height: 640,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn virtual_harness(app: fn() -> Element) -> Harness {
    Harness::with_config(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual))
}

fn item(value: u8, title: String) -> MenuEntry<u8> {
    MenuEntry::Item {
        value,
        title,
        detail: None,
        tile: None,
        trail: MenuTrail::None,
        check: None,
        availability: Availability::Enabled,
    }
}

/// The "Applications" group's rows: never grown or shrunk by the palette itself — the test steps
/// it directly, as a caller-driven activation would (sill's keyboard machine toggling a section
/// without going through the palette's own Enter or click).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Rows {
    Collapsed,
    Grown,
}

static ROWS: GlobalSignal<Rows> = Signal::global(|| Rows::Collapsed);
/// The palette's handle, published once the page mounts: the test marks through it before
/// stepping `ROWS`.
static HANDLE: GlobalSignal<Option<PaletteHandle>> = Signal::global(|| None);

fn groups(rows: Rows) -> PaletteGroups<u8> {
    let (count, label) = match rows {
        Rows::Collapsed => (2, "Show More"),
        Rows::Grown => (6, "Show Less"),
    };
    let apps = (0..count).map(|n| item(n, format!("App {n}"))).collect();
    PaletteGroups(vec![
        // The action never runs (the test steps `ROWS` itself): a real caller-driven activation
        // has nothing for the palette's own `run` to call either.
        PaletteGroup::list("Applications", apps).with_action(label, EventHandler::new(|()| {})),
        PaletteGroup::list("Settings", vec![item(90, "Displays".to_string())]),
    ])
}

#[allow(non_snake_case)]
fn Launcher() -> Element {
    let handle = use_palette_handle();
    use_hook(|| *HANDLE.write() = Some(handle));
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { style: "width:600px;height:600px",
                CommandPalette::<u8> {
                    label: "Launch",
                    placeholder: "Search",
                    query: String::new(),
                    tokens: Vec::new(),
                    groups: groups(ROWS()),
                    empty: "Nothing",
                    oninput: move |_| {},
                    onpick: move |_| {},
                    onclose: move |()| {},
                    host: CommandPaletteHost::Surface,
                    id: "card".to_string(),
                    handle: Some(handle),
                }
            }
        }
    }
}

fn motion_count(harness: &Harness) -> usize {
    harness.count("[*|data-row-motion]")
}

fn settle(harness: &mut Harness) {
    settle_until(harness, |h| motion_count(h) == 0);
    assert_settles_to_zero_frames(harness);
}

/// Mark `group` through the mounted handle, then set `ROWS`, both before the render that reads
/// them: the mark lands in the same result change as the row count.
fn mark_and_set(harness: &mut Harness, group: &str, rows: Rows) {
    let group = group.to_string();
    harness.within(|| {
        HANDLE
            .read()
            .expect("the palette has mounted")
            .mark_group_action(group);
        *ROWS.write() = rows;
    });
    harness.advance(ms(20));
}

/// Set `ROWS` with no mark at all: a caller-driven change the palette never hears an action for.
fn set_unmarked(harness: &mut Harness, rows: Rows) {
    harness.within(|| *ROWS.write() = rows);
    harness.advance(ms(20));
}

fn opened(harness: &mut Harness) {
    harness.advance(ms(50));
    assert_eq!(
        harness.count("#card .ds-menu-item"),
        3,
        "collapsed to start (2 apps, 1 setting)"
    );
    assert_eq!(motion_count(harness), 0);
}

#[test]
fn a_caller_marked_show_more_rises_the_added_rows_like_the_palettes_own_enter() {
    let mut harness = virtual_harness(Launcher);
    opened(&mut harness);

    mark_and_set(&mut harness, "Applications", Rows::Grown);
    assert_eq!(harness.count("#card .ds-menu-item"), 7);
    assert_eq!(
        harness.count("[*|data-row-motion=in]"),
        4,
        "the four added rows rise, marked or not"
    );
    settle(&mut harness);
}

#[test]
fn a_caller_marked_show_less_heals_by_the_removed_rows_height_like_the_palettes_own_enter() {
    let mut harness = virtual_harness(Launcher);
    opened(&mut harness);
    // Grown first, marked, so the added rows' span gets measured (what Show Less heals by):
    // the same setup the palette's own Enter needs before it can play a heal.
    mark_and_set(&mut harness, "Applications", Rows::Grown);
    assert_eq!(harness.count("#card .ds-menu-item"), 7);
    settle(&mut harness);

    mark_and_set(&mut harness, "Applications", Rows::Collapsed);
    assert_eq!(harness.count("#card .ds-menu-item"), 3);
    assert_eq!(harness.count("[*|data-row-motion=heal-from]"), 1);
    let style = harness.attr("#card .ds-menu", "style").unwrap_or_default();
    let dy: f32 = style
        .strip_prefix("--dy:")
        .and_then(|rest| rest.strip_suffix("px"))
        .and_then(|px| px.parse().ok())
        .unwrap_or_else(|| panic!("the list heals by --dy: {style:?}"));
    assert!(dy > 0.0, "a positive heal, not {dy}");
    settle(&mut harness);
}

#[test]
fn an_unmarked_caller_driven_change_plays_nothing() {
    let mut harness = virtual_harness(Launcher);
    opened(&mut harness);

    set_unmarked(&mut harness, Rows::Grown);
    assert_eq!(harness.count("#card .ds-menu-item"), 7);
    assert_eq!(motion_count(&harness), 0, "no mark, no motion");
    settle(&mut harness);
}

#[test]
fn a_mark_for_the_wrong_group_plays_nothing() {
    let mut harness = virtual_harness(Launcher);
    opened(&mut harness);

    // "Settings" never resizes; the mark names a group other than the one that actually grew.
    mark_and_set(&mut harness, "Settings", Rows::Grown);
    assert_eq!(harness.count("#card .ds-menu-item"), 7);
    assert_eq!(
        motion_count(&harness),
        0,
        "the mark named a different group"
    );
    settle(&mut harness);
}
