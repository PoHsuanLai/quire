//! `CollectionView` over 1,000 tiles builds the tiles of the rows near the viewport only, moves
//! its cursor in two dimensions with the arrows, Home and End, extends it with Shift, opens the
//! cursor's tile on Command-Down, jumps to a caption by type-select, clips a long file name from the
//! middle and keeps its extension, and fades a tile the owner removes while it is mounted.

use dioxus::prelude::*;
use ds::base::selection::{Pick, PickHow};
use ds::components::content::icon_source::IconSource;
use ds::components::lists::collection_view::{CollectionItem, CollectionView, Flow, Thumbnail};
use ds::prelude::*;
use ds_harness::{Clock, Harness, HarnessConfig, Input, Viewport};
use std::cell::Cell;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 400,
    height: 400,
    scale_percent: 100,
};
const TILES: u32 = 1_000;
/// The grid's columns, fixed so the test does not depend on the measured width.
const COLUMNS: usize = 4;
/// Past the fade.
const SETTLED: Duration = Duration::from_millis(700);
/// The tile whose name is long enough to be clipped.
const LONG: u32 = 7;

thread_local! {
    /// The tile `onopen` last heard, on this test's thread.
    static OPENED: Cell<Option<usize>> = const { Cell::new(None) };
    /// The last `Pick` `onreach` heard, on this test's thread.
    static REACHED: Cell<Option<Pick>> = const { Cell::new(None) };
}

fn caption(n: u32) -> String {
    match n {
        LONG => "Screenshot 2024-05-14 at 14.05.png".to_owned(),
        _ => format!("{} {n}", ["Apple", "Berry", "Cherry"][n as usize % 3]),
    }
}

#[allow(non_snake_case)]
fn Page() -> Element {
    let mut keys = use_signal(|| (0..TILES).collect::<Vec<u32>>());
    let mut cursor = use_signal(|| None::<usize>);
    let items: Vec<CollectionItem<u32>> = keys()
        .into_iter()
        .map(|n| {
            CollectionItem::new(
                n,
                caption(n),
                Thumbnail::Icon(IconSource::Glyph(Icon::File)),
            )
        })
        .collect();
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "height:300px; width:400px; display:flex; flex-direction:column",
                CollectionView::<u32> {
                    label: "Files",
                    items,
                    flow: Flow::Columns(COLUMNS as u8),
                    cursor: cursor(),
                    onreach: move |pick: Pick| {
                        REACHED.set(Some(pick));
                        cursor.set(Some(pick.row));
                    },
                    onopen: |at| OPENED.set(Some(at)),
                }
            }
            button { class: "remove-3", onclick: move |_| keys.with_mut(|keys| keys.retain(|key| *key != 3)), "remove" }
            p { class: "cursor", "{cursor().map_or(-1, |row| row as i64)}" }
        }
    }
}

fn harness() -> Harness {
    OPENED.set(None);
    REACHED.set(None);
    Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual))
}

fn click(harness: &mut Harness, selector: &str) {
    let at = harness.centre(selector).expect("laid out");
    harness.send(Input::click(at));
}

fn tile(index: usize) -> String {
    format!(".ds-cv-item[data-index=\"{index}\"]")
}

fn cursor_line(harness: &Harness) -> i64 {
    harness
        .text_of(".cursor")
        .and_then(|text| text.parse().ok())
        .expect("the cursor line")
}

fn press(harness: &mut Harness, key: ShortcutKey) {
    harness.send(Input::key(key));
}

#[test]
fn only_the_tiles_of_the_rows_near_the_viewport_are_mounted() {
    let harness = harness();
    let mounted = harness.count(".ds-cv-item");
    assert!(
        (3 * COLUMNS..=7 * COLUMNS).contains(&mounted),
        "{mounted} tiles mounted of {TILES}"
    );
    assert_eq!(harness.attr(&tile(0), "data-index").as_deref(), Some("0"));
    assert_eq!(harness.count(&tile(TILES as usize - 1)), 0);
}

#[test]
fn the_arrows_move_the_cursor_by_a_tile_and_by_a_row_and_home_and_end_go_to_the_ends() {
    let mut harness = harness();
    click(&mut harness, &tile(0));
    assert_eq!(cursor_line(&harness), 0);
    assert_eq!(harness.count(".ds-cv-item[aria-selected=true]"), 1);
    // (key, cursor after)
    let steps = [
        (ShortcutKey::Down, COLUMNS as i64),
        (ShortcutKey::Right, COLUMNS as i64 + 1),
        (ShortcutKey::Up, 1),
        (ShortcutKey::Left, 0),
        (ShortcutKey::Left, 0),
    ];
    for (key, want) in steps {
        press(&mut harness, key);
        assert_eq!(cursor_line(&harness), want, "{key:?}");
    }
    press(&mut harness, ShortcutKey::End);
    assert_eq!(cursor_line(&harness), i64::from(TILES) - 1);
    assert_eq!(
        harness.count(&tile(TILES as usize - 1)),
        1,
        "the cursor is in view"
    );
    press(&mut harness, ShortcutKey::Home);
    assert_eq!(cursor_line(&harness), 0);
}

#[test]
fn shift_with_an_arrow_extends_and_the_plain_arrow_replaces() {
    let mut harness = harness();
    click(&mut harness, &tile(0));
    harness.send(Input::chord(&[ShortcutKey::Shift], ShortcutKey::Down));
    assert_eq!(
        REACHED.get(),
        Some(Pick {
            row: COLUMNS,
            how: PickHow::Extend
        })
    );
    press(&mut harness, ShortcutKey::Right);
    assert_eq!(
        REACHED.get().map(|pick| pick.how),
        Some(PickHow::Replace),
        "a plain arrow replaces"
    );
}

#[test]
fn command_down_opens_the_cursor_tile_return_does_not_and_a_letter_jumps_to_the_next_caption_that_starts_with_it()
 {
    let mut harness = harness();
    click(&mut harness, &tile(0));
    assert_eq!(OPENED.get(), None, "a single click opens nothing");
    press(&mut harness, ShortcutKey::Enter);
    assert_eq!(OPENED.get(), None, "Return is the app's (Finder's Rename)");
    harness.send(Input::chord(&[ShortcutKey::Super], ShortcutKey::Down));
    assert_eq!(OPENED.get(), Some(0));
    // "Apple 0", "Berry 1", "Cherry 2": the next caption starting with c is tile 2.
    press(&mut harness, ShortcutKey::Char('c'));
    assert_eq!(cursor_line(&harness), 2);
    harness.send(Input::chord(&[ShortcutKey::Super], ShortcutKey::Down));
    assert_eq!(OPENED.get(), Some(2));
}

#[test]
fn a_long_name_is_clipped_from_the_middle_and_keeps_its_extension() {
    let harness = harness();
    let shown = harness
        .text_of(&format!("{} .ds-cv-caption", tile(LONG as usize)))
        .expect("the caption");
    assert_eq!(shown, "Screens….05.png");
    assert_eq!(
        harness
            .text_of(&format!("{} .ds-cv-caption", tile(0)))
            .as_deref(),
        Some("Apple 0"),
        "a short name is whole"
    );
}

#[test]
fn a_tile_removed_while_mounted_fades_where_it_stood_and_is_gone_once_it_settles() {
    let mut harness = harness();
    let before = harness.count(".ds-cv-item");
    click(&mut harness, ".remove-3");
    assert_eq!(
        harness
            .attr(".ds-cv-item[data-presence=leaving]", "data-exit")
            .as_deref(),
        Some("fade")
    );
    assert_eq!(
        harness.count(".ds-cv-item"),
        before + 1,
        "it stays while it fades, and the window takes the next tile in"
    );
    harness.advance(SETTLED);
    assert_eq!(harness.count(".ds-cv-item[data-presence=leaving]"), 0);
    assert_eq!(
        harness.count(".ds-cv-item"),
        before,
        "the window is full again"
    );
    assert_eq!(
        harness
            .text_of(&format!("{} .ds-cv-caption", tile(3)))
            .as_deref(),
        Some(caption(4).as_str()),
        "the tile after it took its place"
    );
}
