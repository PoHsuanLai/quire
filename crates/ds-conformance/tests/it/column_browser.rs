//! `ColumnBrowser` over a small tree the page keeps: a click picks a row and the page answers with
//! the next column, the arrows move within a column, Right enters a folder and Left goes back
//! (cutting the columns after the parent), letters jump to a name, Command-Down
//! opens where Return does not, a leaf selection shows the preview slot, a long name
//! is cut from the middle, and a divider drag resizes the columns.

use dioxus::prelude::*;
use ds::base::selection::{Pick, PickHow, RowSelection};
use ds::components::content::icon_source::IconSource;
use ds::components::lists::column_browser::{BrowserColumn, BrowserRow, ColumnBrowser, ColumnPick};
use ds::prelude::*;
use ds_harness::{Clock, Harness, HarnessConfig, Input, Viewport};
use std::cell::Cell;

const VIEW: Viewport = Viewport {
    width: 700,
    height: 300,
    scale_percent: 100,
};

thread_local! {
    /// The pick `onopen` last heard, on this test's thread.
    static OPENED: Cell<Option<ColumnPick>> = const { Cell::new(None) };
}

/// A folder (with its contents) or a file.
struct Node {
    name: &'static str,
    kids: Option<Vec<Node>>,
}

fn tree() -> Vec<Node> {
    let file = |name| Node { name, kids: None };
    vec![
        Node {
            name: "Docs",
            kids: Some(vec![file("a.pdf"), file("Screenshot 2024-05-14 at 14.05.png")]),
        },
        Node {
            name: "Music",
            kids: Some(vec![file("song.mp3")]),
        },
        file("notes.txt"),
    ]
}

fn columns(root: &[Node], path: &[usize]) -> Vec<BrowserColumn<&'static str>> {
    let mut shown = Vec::new();
    let mut level = Some(root);
    for depth in 0.. {
        let Some(nodes) = level else { break };
        let picked = path.get(depth).copied();
        let rows = nodes
            .iter()
            .map(|node| {
                let icon = IconSource::Glyph(Icon::File);
                match node.kids {
                    Some(_) => BrowserRow::branch(node.name, node.name, icon),
                    None => BrowserRow::leaf(node.name, node.name, icon),
                }
            })
            .collect();
        let selection = picked.map_or_else(RowSelection::none, RowSelection::single);
        shown.push(BrowserColumn::new(rows).selected(selection));
        level = picked.and_then(|row| nodes[row].kids.as_deref());
    }
    shown
}

#[allow(non_snake_case)]
fn Page() -> Element {
    let mut path = use_signal(Vec::<usize>::new);
    let root = tree();
    let line = path()
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(",");
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "height:240px; width:660px; display:flex; flex-direction:column",
                ColumnBrowser::<&'static str> {
                    label: "Files",
                    columns: columns(&root, &path()),
                    preview: rsx! {
                        p { class: "preview", "preview" }
                    },
                    column_width: Px(200.0),
                    onpick: move |pick: ColumnPick| {
                        path.with_mut(|path| {
                            path.truncate(pick.column);
                            path.push(pick.pick.row);
                        });
                    },
                    onopen: |pick| OPENED.set(Some(pick)),
                }
            }
            p { class: "path", "{line}" }
        }
    }
}

fn harness() -> Harness {
    OPENED.set(None);
    Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual))
}

fn row(column: usize, at: usize) -> String {
    let item = format!(".ds-colbrowser-item[data-index=\"{at}\"] .ds-row");
    format!(".ds-colbrowser-column[data-column=\"{column}\"] {item}")
}

fn click(harness: &mut Harness, selector: &str) {
    let at = harness.centre(selector).expect("laid out");
    harness.send(Input::click(at));
}

fn picked(harness: &Harness) -> String {
    harness.text_of(".path").unwrap_or_default()
}

fn press(harness: &mut Harness, key: ShortcutKey) {
    harness.send(Input::key(key));
}

#[test]
fn a_click_picks_a_row_and_the_page_answers_with_the_next_column() {
    let mut harness = harness();
    assert_eq!(harness.count(".ds-colbrowser-column"), 1);
    click(&mut harness, &row(0, 0));
    assert_eq!(picked(&harness), "0");
    assert_eq!(harness.count(".ds-colbrowser-column"), 2, "Docs' contents");
    click(&mut harness, &row(0, 1));
    assert_eq!(picked(&harness), "1");
    assert_eq!(harness.count(".ds-colbrowser-column"), 2, "Music's, not Docs'");
    let chevrons = harness.count(".ds-colbrowser-item .ds-row-trailing");
    assert_eq!(chevrons, 2, "a chevron on each folder");
}

#[test]
fn up_down_right_and_left_walk_the_columns_as_finder_does() {
    let mut harness = harness();
    click(&mut harness, &row(0, 0));
    press(&mut harness, ShortcutKey::Down);
    assert_eq!(picked(&harness), "1");
    press(&mut harness, ShortcutKey::Down);
    assert_eq!(picked(&harness), "2", "notes.txt, a file");
    press(&mut harness, ShortcutKey::Down);
    assert_eq!(picked(&harness), "2", "stops at the end");
    press(&mut harness, ShortcutKey::Right);
    assert_eq!(picked(&harness), "2", "a file has no column to enter");
    press(&mut harness, ShortcutKey::Up);
    press(&mut harness, ShortcutKey::Right);
    assert_eq!(picked(&harness), "1,0", "Music's first row");
    assert_eq!(harness.count(".ds-colbrowser-column"), 2);
    press(&mut harness, ShortcutKey::Left);
    assert_eq!(picked(&harness), "1", "back in the parent, the columns after it cut");
}

#[test]
fn a_selected_file_shows_the_preview_and_a_folder_does_not() {
    let mut harness = harness();
    click(&mut harness, &row(0, 2));
    assert_eq!(harness.count(".ds-colbrowser-preview"), 1);
    click(&mut harness, &row(0, 0));
    assert_eq!(harness.count(".ds-colbrowser-preview"), 0);
}

#[test]
fn command_down_and_a_double_click_open_where_return_does_not_and_a_letter_jumps() {
    let mut harness = harness();
    click(&mut harness, &row(0, 0));
    press(&mut harness, ShortcutKey::Enter);
    assert_eq!(OPENED.get(), None, "Return is the app's");
    harness.send(Input::chord(&[ShortcutKey::Super], ShortcutKey::Down));
    assert_eq!(
        OPENED.get(),
        Some(ColumnPick {
            column: 0,
            pick: Pick {
                row: 0,
                how: PickHow::Replace
            }
        })
    );
    press(&mut harness, ShortcutKey::Char('n'));
    assert_eq!(picked(&harness), "2", "notes.txt starts with n");
}

#[test]
fn a_long_name_is_cut_from_the_middle_and_keeps_its_extension() {
    let mut harness = harness();
    click(&mut harness, &row(0, 0));
    let shown = harness
        .text_of(&format!("{} .ds-name-text", row(1, 1)))
        .expect("the name");
    assert!(shown.contains('…') && shown.ends_with(".png"), "{shown}");
}

#[test]
fn dragging_a_divider_resizes_the_columns() {
    let mut harness = harness();
    click(&mut harness, &row(0, 0));
    let width = |harness: &Harness| {
        harness
            .attr(".ds-colbrowser-column", "style")
            .expect("a width")
    };
    assert_eq!(width(&harness), "width:200px");
    let from = harness.centre(".ds-colbrowser-divider").expect("a divider");
    let to = Point {
        x: Px(from.x.0 + 40.0),
        y: from.y,
    };
    harness.send(Input::drag(from, to, 4));
    assert_eq!(width(&harness), "width:240px");
}
