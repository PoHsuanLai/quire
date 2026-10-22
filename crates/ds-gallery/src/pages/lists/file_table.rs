//! VirtualTable as Finder's list view: the first column is an outline (a disclosure triangle,
//! Option-click opens a whole folder, Right and Left on the cursor's row open and close it) of
//! name cells (a glyph and a name cut from the middle), and a drag from empty space below the rows
//! draws a rubber band that selects the rows it touches. The page keeps which folders are open, as
//! an app does.

use crate::pages::Section;
use dioxus::prelude::*;
use ds::base::selection::{Pick, RowSelection};
use ds::components::content::icon_source::IconSource;
use ds::components::controls::scroller::handle::use_scroller;
use ds::components::lists::name_cell::NameCell;
use ds::components::lists::table::model::TableColumn;
use ds::components::lists::virtual_table::{OutlineRow, OutlineToggle, ToggleReach, VirtualTable};
use ds::prelude::*;
use ds_core::vocab::Shown;

/// One entry of the tree, in the order Finder lists it: `depth` folders deep, a folder or not.
struct Entry {
    name: &'static str,
    depth: u16,
    folder: bool,
}

const fn entry(name: &'static str, depth: u16, folder: bool) -> Entry {
    Entry {
        name,
        depth,
        folder,
    }
}

const TREE: [Entry; 11] = [
    entry("Projects", 0, true),
    entry("Website redesign with a long name.sketch", 1, false),
    entry("Archive", 1, true),
    entry("2025 budget.xlsx", 2, false),
    entry("2024 budget.xlsx", 2, false),
    entry("Notes", 1, false),
    entry("Photos", 0, true),
    entry("Holiday", 1, true),
    entry("IMG_0001.png", 2, false),
    entry("IMG_0002.png", 2, false),
    entry("README", 0, false),
];

/// The tree indexes of the entries that show, given the folders in `open`.
fn showing(open: &[usize]) -> Vec<usize> {
    let mut shown = Vec::new();
    let mut closed_at: Option<u16> = None;
    for (at, entry) in TREE.iter().enumerate() {
        if closed_at.is_some_and(|depth| entry.depth > depth) {
            continue;
        }
        closed_at = None;
        shown.push(at);
        if entry.folder && !open.contains(&at) {
            closed_at = Some(entry.depth);
        }
    }
    shown
}

/// The folders under `at`, `at` itself included.
fn folders_under(at: usize) -> Vec<usize> {
    let depth = TREE[at].depth;
    std::iter::once(at)
        .chain(
            TREE[at + 1..]
                .iter()
                .enumerate()
                .take_while(|(_, entry)| entry.depth > depth)
                .filter(|(_, entry)| entry.folder)
                .map(|(offset, _)| at + 1 + offset),
        )
        .collect()
}

/// `open` after `toggle`, the row being an index into `shown`.
fn toggled(open: Vec<usize>, shown: &[usize], toggle: OutlineToggle) -> Vec<usize> {
    let Some(&at) = shown.get(toggle.row) else {
        return open;
    };
    let reached = match toggle.reach {
        ToggleReach::Row => vec![at],
        ToggleReach::Subtree => folders_under(at),
    };
    let kept: Vec<usize> = open
        .into_iter()
        .filter(|held| !reached.contains(held))
        .collect();
    match toggle.shown {
        Shown::Visible => kept.into_iter().chain(reached).collect(),
        Shown::Hidden => kept,
    }
}

/// The table, with the folders it has open.
#[component]
pub fn FileTableGallery() -> Element {
    let scroller = use_scroller();
    let mut open = use_signal(|| vec![0usize]);
    let mut chosen = use_signal(RowSelection::none);
    let shown = showing(&open());
    let columns = vec![
        TableColumn::new(0u8, "Name", Px(260.0)),
        TableColumn::new(1u8, "Kind", Px(120.0)),
    ];
    let rows = shown.len();
    let listed = shown.clone();
    let outline = Callback::new(move |row: usize| {
        let entry = &TREE[listed[row]];
        match entry.folder {
            true => OutlineRow::branch(
                entry.depth,
                if open.read().contains(&listed[row]) {
                    Shown::Visible
                } else {
                    Shown::Hidden
                },
            ),
            false => OutlineRow::leaf(entry.depth),
        }
    });
    let named = shown.clone();
    let cell = Callback::new(move |(row, column): (usize, usize)| {
        let entry = &TREE[named[row]];
        match column {
            0 => rsx! {
                NameCell {
                    icon: IconSource::Glyph(if entry.folder { Icon::Folder } else { Icon::File }),
                    name: entry.name,
                    room: Px(200.0 - 16.0 * f32::from(entry.depth)),
                }
            },
            _ => rsx! { if entry.folder { "Folder" } else { "Document" } },
        }
    });
    let toggling = shown.clone();
    rsx! {
        Section {
            title: "VirtualTable, outline",
            note: "Finder's list view. The first column is an outline: a triangle opens and closes a folder, Option-click (or Option with Right) opens everything under it, Right and Left do it from the keyboard. The names are cut from the middle to the column's width. A drag from the empty space below the rows draws a rubber band over the rows it touches; Shift or Command adds to the selection, and a click there clears it.",
            div { class: "g-list", style: "width:420px; height:300px; display:flex; flex-direction:column",
                VirtualTable::<u8> {
                    label: "Files",
                    columns,
                    rows,
                    cell,
                    scroller,
                    on_sort: |_| {},
                    look: TableLook::Striped,
                    cursor: chosen().cursor(),
                    selection: Some(chosen()),
                    onreach: move |pick: Pick| chosen.set(chosen().picked(pick)),
                    onmarquee: move |next: RowSelection| chosen.set(next),
                    outline,
                    ontoggle: move |toggle: OutlineToggle| {
                        open.set(toggled(open(), &toggling, toggle));
                    },
                }
            }
        }
    }
}
