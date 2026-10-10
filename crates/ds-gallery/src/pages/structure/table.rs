//! Table: sortable, resizable columns over selectable rows, at 24 and at 44.

use crate::pages::Section;
use dioxus::prelude::*;
use ds::components::lists::row::size::RowSize;
use ds::components::lists::table::model::{CellAlign, Sort, SortDirection, TableColumn, TableRow};
use ds::components::lists::table::rules::ColumnRules;
use ds::components::lists::table::view::Table;
use ds::prelude::*;

/// One file the table lists.
#[derive(Debug, Clone, Copy, PartialEq)]
struct File {
    name: &'static str,
    kind: &'static str,
    kilobytes: u32,
    modified: &'static str,
}

const FILES: [File; 7] = [
    File {
        name: "Quarterly report.pdf",
        kind: "PDF",
        kilobytes: 2480,
        modified: "Today, 09:41",
    },
    File {
        name: "Notes from the sync",
        kind: "Text",
        kilobytes: 12,
        modified: "Yesterday",
    },
    File {
        name: "Site photos",
        kind: "Folder",
        kilobytes: 0,
        modified: "Sep 22",
    },
    File {
        name: "Invoice 2041.xlsx",
        kind: "Sheet",
        kilobytes: 88,
        modified: "Sep 18",
    },
    File {
        name: "Onboarding.key",
        kind: "Slides",
        kilobytes: 14200,
        modified: "Sep 3",
    },
    File {
        name: "backup.tar.zst",
        kind: "Archive",
        kilobytes: 501000,
        modified: "Aug 30",
    },
    File {
        name: "README",
        kind: "Text",
        kilobytes: 4,
        modified: "Aug 12",
    },
];

/// The columns, by id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Column {
    Name,
    Kind,
    Size,
    Modified,
}

fn columns() -> Vec<TableColumn<Column>> {
    vec![
        TableColumn::new(Column::Name, "Name", Px(220.0)),
        TableColumn::new(Column::Kind, "Kind", Px(110.0)),
        TableColumn::new(Column::Size, "Size", Px(100.0)).aligned(CellAlign::Trailing),
        TableColumn::new(Column::Modified, "Date Modified", Px(140.0)),
    ]
}

/// The files ordered by `sort`.
fn ordered(sort: &Sort<Column>) -> Vec<File> {
    let mut files = FILES.to_vec();
    files.sort_by(|a, b| match sort.column {
        Column::Name => a.name.cmp(b.name),
        Column::Kind => a.kind.cmp(b.kind),
        Column::Size => a.kilobytes.cmp(&b.kilobytes),
        Column::Modified => std::cmp::Ordering::Equal,
    });
    if sort.direction == SortDirection::Descending {
        files.reverse();
    }
    files
}

fn size_text(kilobytes: u32) -> String {
    match kilobytes {
        0 => "--".to_owned(),
        1..1000 => format!("{kilobytes} KB"),
        _ => format!("{:.1} MB", f64::from(kilobytes) / 1000.0),
    }
}

/// The rows of `files`.
fn rows(files: &[File]) -> Vec<TableRow<&'static str>> {
    files
        .iter()
        .map(|file| {
            TableRow::new(
                file.name,
                file.name,
                vec![
                    rsx! {
                        span { class: "g-row",
                            IconView { source: if file.kind == "Folder" { Icon::Folder.into() } else { Icon::File.into() } }
                            "{file.name}"
                        }
                    },
                    rsx! { "{file.kind}" },
                    rsx! { "{size_text(file.kilobytes)}" },
                    rsx! { "{file.modified}" },
                ],
            )
        })
        .collect()
}

/// The Table section.
#[component]
pub fn TableSection() -> Element {
    let mut sort = use_signal(|| Sort {
        column: Column::Name,
        direction: SortDirection::Ascending,
    });
    let mut chosen = use_signal(|| vec!["Notes from the sync"]);
    let mut cursor = use_signal(|| Some("Notes from the sync"));
    let files = ordered(&sort());
    rsx! {
        Section { title: "Table", note: "NSTableView: a press on a header sorts by it (again to flip) and the indicator follows; drag the edge between two headers to resize a column, held at its least, the last column taking the rest; rows are not striped; the arrows move the selection; the third table draws a hairline between columns (ColumnRules::Hairline), and a VirtualTable also draws one under the header once its rows scroll.",
            div { class: "g-list", style: "width:620px",
                Table::<&'static str, Column> {
                    label: "Files",
                    columns: columns(),
                    rows: rows(&files),
                    sort: Some(sort()),
                    on_sort: move |next| sort.set(next),
                    selection: chosen(),
                    cursor: cursor(),
                    onselect: move |key| {
                        chosen.set(vec![key]);
                        cursor.set(Some(key));
                    },
                }
            }
            div { class: "g-list", style: "width:620px",
                Table::<&'static str, Column> {
                    label: "Files, settings density",
                    columns: columns(),
                    rows: rows(&files[..3]),
                    sort: Some(Sort { column: Column::Size, direction: SortDirection::Descending }),
                    on_sort: |_| {},
                    selection: vec![],
                    size: RowSize::Settings,
                    onselect: |_| {},
                }
            }
            div { class: "g-list", style: "width:620px",
                Table::<&'static str, Column> {
                    label: "Files, hairline column rules",
                    columns: columns(),
                    rows: rows(&files[..4]),
                    sort: Some(Sort { column: Column::Name, direction: SortDirection::Ascending }),
                    on_sort: |_| {},
                    selection: vec![],
                    rules: ColumnRules::Hairline,
                    onselect: |_| {},
                }
            }
        }
    }
}
