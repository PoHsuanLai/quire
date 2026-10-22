//! ColumnBrowser: Finder's column view over a small tree the page keeps, as an app keeps its
//! folders. A pick in a column cuts the columns after it and shows the picked folder's contents
//! in the next; a file ends the columns and fills the preview. Drag a divider to resize the
//! columns.

use crate::pages::Section;
use dioxus::prelude::*;
use ds::base::selection::RowSelection;
use ds::components::content::icon_source::IconSource;
use ds::components::lists::column_browser::{
    BrowserColumn, BrowserRow, BrowserRowKind, ColumnBrowser, ColumnPick,
};
use ds::prelude::*;

/// A folder (with its contents) or a file.
struct Node {
    name: &'static str,
    kids: Option<Vec<Node>>,
}

fn file(name: &'static str) -> Node {
    Node { name, kids: None }
}

fn folder(name: &'static str, kids: Vec<Node>) -> Node {
    Node {
        name,
        kids: Some(kids),
    }
}

fn tree() -> Vec<Node> {
    vec![
        folder(
            "Projects",
            vec![
                folder(
                    "Website redesign",
                    vec![
                        file("Screenshot 2024-05-14 at 14.05.png"),
                        file("Brief.pdf"),
                    ],
                ),
                folder("Archive", vec![file("2025 budget.xlsx")]),
                file("Notes.txt"),
            ],
        ),
        folder("Photos", vec![file("Beach.jpg"), file("Hills.jpg")]),
        file("Read me.md"),
    ]
}

/// The rows of one level of the tree.
fn rows(nodes: &[Node]) -> Vec<BrowserRow<&'static str>> {
    nodes
        .iter()
        .map(|node| match node.kids {
            Some(_) => BrowserRow::branch(node.name, node.name, IconSource::Glyph(Icon::Folder)),
            None => BrowserRow::leaf(node.name, node.name, IconSource::Glyph(Icon::File)),
        })
        .collect()
}

/// The columns the picked `path` (one row per column) shows: a folder picked in the last column
/// adds its contents as a column with nothing selected.
fn columns(root: &[Node], path: &[usize]) -> Vec<BrowserColumn<&'static str>> {
    let mut shown = Vec::new();
    let mut level = Some(root);
    for depth in 0.. {
        let Some(nodes) = level else { break };
        let picked = path.get(depth).copied().filter(|row| *row < nodes.len());
        let selection = picked.map_or_else(RowSelection::none, RowSelection::single);
        shown.push(BrowserColumn::new(rows(nodes)).selected(selection));
        level = picked.and_then(|row| nodes[row].kids.as_deref());
    }
    shown
}

/// The column browser and the line that says what was opened.
#[component]
pub fn ColumnBrowserGallery() -> Element {
    let mut path = use_signal(|| vec![0usize, 0]);
    let mut opened = use_signal(String::new);
    let root = tree();
    let shown = columns(&root, &path());
    let last = shown.last().and_then(|column| {
        let row = column.selection.cursor()?;
        let item = column.rows.get(row)?;
        (item.kind == BrowserRowKind::Leaf).then(|| item.name.clone())
    });
    let preview = last.map(|name| {
        rsx! {
            p { class: "g-note", "A preview of {name} would fill this space." }
        }
    });
    let said = match opened().as_str() {
        "" => "Nothing opened yet.".to_owned(),
        name => format!("Opened {name}."),
    };
    rsx! {
        Section {
            title: "ColumnBrowser",
            note: "Finder's column view. Click a row, or use Up and Down within a column, Right to enter a folder and Left to go back; letters jump to a name. Command-Down or a double-click opens. A selected file ends the columns and shows the preview slot. The divider on a column's right edge drags to resize every column.",
            div { class: "g-row g-row-top",
                div { class: "g-list", style: "width:620px; height:220px; display:flex; flex-direction:column",
                    ColumnBrowser::<&'static str> {
                        label: "Files",
                        columns: shown.clone(),
                        preview,
                        column_width: Px(180.0),
                        onpick: move |pick: ColumnPick| {
                            path.with_mut(|path| {
                                path.truncate(pick.column);
                                path.push(pick.pick.row);
                            });
                        },
                        onopen: {
                            let shown = shown.clone();
                            move |pick: ColumnPick| {
                                if let Some(row) = shown.get(pick.column).and_then(|c| c.rows.get(pick.pick.row)) {
                                    opened.set(row.name.clone());
                                }
                            }
                        },
                    }
                }
                p { class: "g-note", "{said}" }
            }
        }
    }
}
