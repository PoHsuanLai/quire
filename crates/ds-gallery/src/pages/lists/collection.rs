//! CollectionView: a dozen tiles of Finder's icon view. A badge (tag dots) sits under some
//! captions, a right click selects the tile and names the one a menu would open for, and a drag
//! from empty space draws a rubber band that selects the tiles it touches.

use crate::pages::Section;
use crate::pages::content::picture::wide_image;
use dioxus::prelude::*;
use ds::base::selection::RowSelection;
use ds::components::content::file_icon::FileIconChoice;
use ds::components::content::icon_source::IconSource;
use ds::components::lists::collection_view::{
    CollectionExtent, CollectionItem, CollectionSize, CollectionView, Flow, ThumbAspect, Thumbnail,
};
use ds::prelude::*;

/// How many tiles the grid has.
const TILES: usize = 12;
/// Every this many tiles carries tag dots.
const TAGGED: usize = 3;

/// The tile's name.
fn name(at: usize) -> String {
    format!("Document {}.txt", at + 1)
}

/// Two tag dots, in the accent and in the quiet ink.
fn dots() -> Element {
    let dot = |colour: &str| {
        format!("width:8px; height:8px; border-radius:50%; background:var({colour}); margin:0 1px")
    };
    rsx! {
        span { style: dot("--accent") }
        span { style: dot("--ink-faint") }
    }
}

/// Six wallpapers as landscape tiles.
fn wallpapers() -> Vec<CollectionItem<usize>> {
    (0..6)
        .map(|at| {
            CollectionItem::new(
                at,
                format!("Wallpaper {}", at + 1),
                Thumbnail::Image(wide_image()),
            )
        })
        .collect()
}

/// Folders and files drawn with the standard file icon.
fn files() -> Vec<CollectionItem<usize>> {
    let page = |kind: FileKind, ext: &str| FileIconChoice::Standard {
        kind,
        label: ExtLabel::from_extension(ext),
    };
    let choices = [
        ("Documents", FileIconChoice::Folder(FolderKind::Documents)),
        ("Notes.txt", page(FileKind::Document, "txt")),
        ("Report.pdf", page(FileKind::Pdf, "pdf")),
        ("Budget.xlsx", page(FileKind::Spreadsheet, "xlsx")),
    ];
    choices
        .into_iter()
        .enumerate()
        .map(|(at, (name, choice))| CollectionItem::new(at, name, Thumbnail::FileIcon(choice)))
        .collect()
}

/// The collection view and the line that says what its menu was asked for.
#[component]
pub fn CollectionGallery() -> Element {
    let mut cursor = use_signal(|| None::<usize>);
    let mut menu = use_signal(|| None::<usize>);
    let mut chosen = use_signal(RowSelection::none);
    let items: Vec<CollectionItem<usize>> = (0..TILES)
        .map(|at| {
            let item =
                CollectionItem::new(at, name(at), Thumbnail::Icon(IconSource::Glyph(Icon::File)));
            match at % TAGGED {
                0 => item.badged(dots()),
                _ => item,
            }
        })
        .collect();
    let asked = menu().map_or("No menu asked for yet.".to_owned(), |at| {
        format!("A menu was asked for {}.", name(at))
    });
    rsx! {
        Section {
            title: "CollectionView",
            note: "Tiles of a thumbnail over a one-line caption. Every third tile carries a badge centred under its caption (Finder's tag dots) in the room the tile already keeps there, so the rows stay the same height. A right click selects the tile unless it is selected, then asks for its menu with the tile's index; a right click on empty space is the page's. A drag from empty space draws a rubber band in the accent tint and selects the tiles it touches (Shift or Command adds to the selection); a click on empty space clears it.",
            div { class: "g-row g-row-top",
                div { class: "g-list", style: "width:420px; height:300px; display:flex; flex-direction:column",
                    CollectionView::<usize> {
                        label: "Documents",
                        items,
                        flow: Flow::Columns(4),
                        cursor: cursor(),
                        selection: Some(chosen()),
                        onselect: move |at| {
                            cursor.set(Some(at));
                            chosen.set(RowSelection::single(at));
                        },
                        onmarquee: move |next: RowSelection| {
                            cursor.set(next.cursor());
                            chosen.set(next);
                        },
                        oncontextmenu: move |(at, _): (usize, MouseEvent)| menu.set(Some(at)),
                    }
                }
                p { class: "g-note", "{asked}" }
            }
        }
        Section {
            title: "CollectionView, landscape and sized to its rows",
            note: "aspect: Landscape draws each thumbnail 16:10 with its picture covering the box (a wallpaper keeps its shape), and the row shortens to match. extent: Content makes the view as tall as all its rows, for a page that scrolls; the box around it here has no height of its own.",
            div { class: "g-list", style: "width:440px",
                CollectionView::<usize> {
                    label: "Wallpapers",
                    items: wallpapers(),
                    size: CollectionSize::Large,
                    aspect: ThumbAspect::Landscape,
                    extent: CollectionExtent::Content,
                    flow: Flow::Columns(3),
                }
            }
        }
        Section {
            title: "CollectionView, file icons",
            note: "Thumbnail::FileIcon draws a file's icon (a folder, a standard page with its extension) whole at the thumbnail's size, with no frame of the tile's.",
            div { class: "g-list g-stage-pad", style: "width:440px",
                CollectionView::<usize> {
                    label: "Files",
                    items: files(),
                    size: CollectionSize::Large,
                    extent: CollectionExtent::Content,
                    flow: Flow::Columns(4),
                }
            }
        }
    }
}
