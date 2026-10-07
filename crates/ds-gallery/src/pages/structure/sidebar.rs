//! Sidebar: a source list at each sidebar size, with a header and section titles.

use crate::pages::{Section, Specimen};
use dioxus::prelude::*;
use ds::components::chrome::sidebar::Sidebar;
use ds::components::chrome::sidebar_model::SidebarSection;
use ds::components::forms::icon_tile::TileFace;
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;
use ds::style::tokens::control_size::SidebarSize;
use ds::style::tokens::hex::Hex;
use ds_core::vocab::RowState;

/// The places, with a heading before each group.
fn places(here: &'static str, onselect: EventHandler<&'static str>) -> Vec<ListItem<&'static str>> {
    let place = move |key: &'static str, title: &'static str, leading: RowLeading, badge: u32| {
        ListItem::row(
            key,
            title,
            rsx! {
                Row {
                    title: TextLine::from(title),
                    leading,
                    accessory: if badge > 0 { Accessory::Badge(badge) } else { Accessory::None },
                    state: RowState {
                        selection: if here == key { Selection::Selected } else { Selection::Unselected },
                        ..RowState::default()
                    },
                    onclick: move |_| onselect.call(key),
                }
            },
        )
    };
    vec![
        ListItem::heading(
            "h-favourites",
            rsx! { SectionHeader { title: "Favourites" } },
        ),
        place("inbox", "Inbox", RowLeading::Icon(Icon::Inbox), 4),
        place("starred", "Starred", RowLeading::Icon(Icon::Star), 0),
        place("sent", "Sent", RowLeading::Icon(Icon::Send), 0),
        ListItem::heading("h-folders", rsx! { SectionHeader { title: "Folders" } }),
        place(
            "archive",
            "Archive",
            RowLeading::Tile(TileFace::Glyph(Icon::Archive, Hex([0x8e, 0x8e, 0x93]))),
            0,
        ),
        place(
            "trash",
            "Trash",
            RowLeading::Tile(TileFace::Glyph(Icon::Trash, Hex([0xff, 0x3b, 0x30]))),
            0,
        ),
    ]
}

/// The Sidebar section.
#[component]
pub fn SidebarListSection() -> Element {
    let mut here = use_signal(|| "inbox");
    let mut query = use_signal(String::new);
    rsx! {
        Section { title: "Sidebar", note: "NSSplitViewItem sidebar: a source list on the sidebar ground at the appearance.sidebar_size row height (24, 28, 32) under an optional header (the Folders rows lead with an IconTile that follows the size: 20, 24, 28); the selection is the accent while the list holds the keyboard and grey when it does not.",
            div { class: "g-row g-row-top",
                for size in SidebarSize::ALL.iter().copied() {
                    Specimen { key: "{size.slug()}", name: format!("SidebarSize::{}", size.label()),
                        div { class: "g-side-frame",
                            Sidebar::<&'static str> {
                                label: "Mail",
                                size,
                                cursor: Some(here()),
                                sections: vec![SidebarSection::List(places(here(), EventHandler::new(move |key| here.set(key))))],
                                onselect: move |key| here.set(key),
                                header: rsx! {
                                    TextField { label: "Search", value: query(), kind: FieldKind::Search, placeholder: "Search", size: ControlSize::Regular, oninput: move |next| query.set(next) }
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}
