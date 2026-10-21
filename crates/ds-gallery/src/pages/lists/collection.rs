//! CollectionView: a dozen tiles of Finder's icon view. A badge (tag dots) sits under some
//! captions, and a right click selects the tile and names the one a menu would open for.

use crate::pages::Section;
use dioxus::prelude::*;
use ds::components::content::icon_source::IconSource;
use ds::components::lists::collection_view::{CollectionItem, CollectionView, Flow, Thumbnail};
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

/// The collection view and the line that says what its menu was asked for.
#[component]
pub fn CollectionGallery() -> Element {
    let mut cursor = use_signal(|| None::<usize>);
    let mut menu = use_signal(|| None::<usize>);
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
            note: "Tiles of a thumbnail over a one-line caption. Every third tile carries a badge centred under its caption (Finder's tag dots) in the room the tile already keeps there, so the rows stay the same height. A right click selects the tile unless it is selected, then asks for its menu with the tile's index; a right click on empty space is the page's.",
            div { class: "g-row g-row-top",
                div { class: "g-list", style: "width:420px; height:300px; display:flex; flex-direction:column",
                    CollectionView::<usize> {
                        label: "Documents",
                        items,
                        flow: Flow::Columns(4),
                        cursor: cursor(),
                        onselect: move |at| cursor.set(Some(at)),
                        oncontextmenu: move |(at, _): (usize, MouseEvent)| menu.set(Some(at)),
                    }
                }
                p { class: "g-note", "{asked}" }
            }
        }
    }
}
