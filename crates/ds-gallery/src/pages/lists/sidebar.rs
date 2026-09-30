//! A source list: rows at the sidebar's size on the Space's frame, a collapsible header, counts
//! as badges, a folder tree as an outline, the rows that close, and drop places lit under a
//! dragged thread (design/30 sections 2.6 and 2.11).

use crate::pages::{Section, Specimen};
use dioxus::prelude::*;
use ds::{
    Accessory, AvatarFace, AvatarShape, AvatarSize, AvatarTone, Common, DataAttr, DataName,
    DropState, Icon, List, ListItem, ListStyle, Outline, PersonHue, Propagation, Row, RowLeading,
    RowState, SectionHeader, Selection, Shown, SidebarSize,
};
use ds::{Bezel, Button, ImagePosition};

/// The places a sidebar offers.
const PLACES: [(Icon, &str, u32); 4] = [
    (Icon::Inbox, "Inbox", 12),
    (Icon::Star, "Starred", 0),
    (Icon::Clock, "Snoozed", 2),
    (Icon::Send, "Sent", 0),
];

/// The badge a count draws; none at zero.
fn count(value: u32) -> Accessory {
    match value {
        0 => Accessory::None,
        value => Accessory::Badge(value),
    }
}

/// `data-place="<place>"`, for a drag that reads the place off the element under the pointer.
fn place(name: &str) -> Common {
    Common {
        data: DataName::parse("place")
            .map(|attribute| vec![DataAttr::new(attribute, name)])
            .unwrap_or_default(),
        ..Common::default()
    }
}

/// The sidebar page: places, Today rows, drop places and a folder tree, at each size.
#[component]
pub fn Sidebar() -> Element {
    rsx! {
        Places {}
        DropPlaces {}
        FolderTree {}
    }
}

/// Places under a collapsible header, and Today rows that close, at the three sidebar sizes.
#[component]
fn Places() -> Element {
    let mut here = use_signal(|| "Inbox");
    let mut open = use_signal(|| Shown::Visible);
    let mut today = use_signal(|| vec!["Dana Okafor", "Priya Raman"]);
    let person = |name: &str| AvatarFace {
        initial: name.chars().next().unwrap_or('?'),
        size: AvatarSize::Size18,
        tone: AvatarTone::Person(PersonHue::of(name)),
        shape: AvatarShape::Round,
    };
    let mut items: Vec<ListItem<&'static str>> = vec![ListItem::heading(
        "Favourites",
        rsx! {
            SectionHeader { title: "Favourites", collapse: Some((open(), EventHandler::new(move |to| open.set(to)))) }
        },
    )];
    if open() == Shown::Visible {
        items.extend(PLACES.into_iter().map(|(icon, label, unread)| {
            ListItem::row(
                label,
                label,
                rsx! {
                    Row {
                        title: label,
                        leading: RowLeading::Icon(icon),
                        accessory: count(unread),
                        state: RowState { selection: Selection::of(&here(), &label), ..RowState::default() },
                        onclick: move |_| here.set(label),
                    }
                },
            )
        }));
    }
    items.push(ListItem::heading(
        "Today",
        rsx! { SectionHeader { title: "Today" } },
    ));
    items.extend(today().into_iter().map(|name| {
        ListItem::row(
            name,
            name,
            rsx! {
                Row {
                    title: name,
                    leading: RowLeading::Avatar(person(name)),
                    accessory: Accessory::Slot(rsx! {
                        Button {
                            bezel: Bezel::Toolbar, image: ImagePosition::Only,
                            icon: Icon::X,
                            label: format!("Close {name}"),
                            propagation: Propagation::Stop,
                            onclick: move |_| today.with_mut(|today| today.retain(|seen| *seen != name)),
                        }
                    }),
                }
            },
        )
    }));
    rsx! {
        Section {
            title: "List: a source list",
            note: "Rows take the sidebar's size (Small 24, Medium 28, Large 32). A header collapses its group; a Today row closes with its own button (named for its row) and the rows below close the gap.",
            div { class: "g-row g-row-top",
                for (name , size) in [("Small", SidebarSize::Small), ("Medium", SidebarSize::Medium), ("Large", SidebarSize::Large)] {
                    Specimen { key: "{name}", name: "SidebarSize::{name}",
                        div { class: "g-side",
                            List::<&'static str> {
                                label: "Places",
                                style: ListStyle::SourceList,
                                sidebar: size,
                                cursor: Some(here()),
                                items: items.clone(),
                                onselect: move |key: &'static str| here.set(key),
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The places a thread can be dropped on.
const DROP_PLACES: [(&str, &str, Icon); 3] = [
    ("inbox", "Inbox", Icon::Inbox),
    ("archive", "Archive", Icon::Archive),
    ("label:7", "Invoices", Icon::Tag),
];

/// Places that light as the drop target under the pointer, from their own pointer hooks.
#[component]
fn DropPlaces() -> Element {
    // Posed with Archive lit, as a drag over it would leave it.
    let mut over = use_signal(|| Some("archive"));
    let mut dropped = use_signal(|| "nothing yet".to_string());
    rsx! {
        Section {
            title: "Row: places as drop targets",
            note: "A place is named with data-place through Common; onpointerenter, onpointerleave and onpointerup hand the pointer to the caller, which sets drop: DropState::Target on the place under it (accent-soft, 1.045). Release over one to drop.",
            div { class: "g-side", style: "width:220px",
                List::<&'static str> {
                    label: "Drop places",
                    style: ListStyle::SourceList,
                    items: DROP_PLACES
                        .into_iter()
                        .map(|(id, label, icon)| {
                            ListItem::row(
                                id,
                                label,
                                rsx! {
                                    Row {
                                        title: label,
                                        leading: RowLeading::Icon(icon),
                                        state: RowState { drop: if over() == Some(id) { DropState::Target } else { DropState::Idle }, ..RowState::default() },
                                        common: place(id),
                                        onpointerenter: move |_| over.set(Some(id)),
                                        onpointerleave: move |_| over.set(None),
                                        onpointerup: move |_| dropped.set(id.to_string()),
                                    }
                                },
                            )
                        })
                        .collect::<Vec<_>>(),
                }
            }
            p { class: "g-note", "Dropped on: {dropped}" }
        }
    }
}

/// `data-folder="<path>"`.
fn folder(path: &str) -> Vec<DataAttr> {
    DataName::parse("folder")
        .map(|name| vec![DataAttr::new(name, path)])
        .unwrap_or_default()
}

/// A folder's drop state: the one under the pointer is the target, every other one accepts.
fn drop_on(over: Option<&'static str>, path: &'static str) -> DropState {
    match over {
        Some(at) if at == path => DropState::Target,
        _ => DropState::Accepts,
    }
}

/// A folder tree mid-drag: Projects open with two subfolders, Archive closed, Archive lit.
#[component]
fn FolderTree() -> Element {
    // Posed with Archive under the pointer, as a drag over it would leave it.
    let mut over = use_signal(|| Some("Archive"));
    let mut said = use_signal(|| "nothing yet".to_string());
    let mut current = use_signal(|| "INBOX/Projects/Quire");
    let mut projects = use_signal(|| Shown::Visible);
    let mut archive = use_signal(|| Shown::Hidden);
    let more = move |path: &'static str| {
        Accessory::Slot(rsx! {
            Button {
                common: Common { data: folder(path), ..Common::default() },
                bezel: Bezel::Toolbar,
                image: ImagePosition::Only,
                icon: Icon::Ellipsis,
                label: "Actions for {path}",
                propagation: Propagation::Stop,
                onclick: move |_| said.set(format!("the menu for {path}")),
            }
        })
    };
    let leaf = move |path: &'static str, label: &'static str| {
        rsx! {
            Row {
                title: label,
                leading: RowLeading::Icon(Icon::Folder),
                outline: Outline::Leaf,
                accessory: more(path),
                state: RowState { selection: Selection::of(&current(), &path), drop: drop_on(over(), path), ..RowState::default() },
                common: place(path),
                onclick: move |_| current.set(path),
                onpointerenter: move |_| over.set(Some(path)),
                onpointerleave: move |_| over.set(None),
                onpointerup: move |_| said.set(format!("dropped on {path}")),
            }
        }
    };
    let branch = move |path: &'static str,
                       label: &'static str,
                       icon: Icon,
                       shown: Shown,
                       toggle: EventHandler<Shown>,
                       unread: u32,
                       children: Element| {
        rsx! {
            Row {
                title: label,
                leading: RowLeading::Icon(icon),
                outline: Outline::Branch(shown),
                on_toggle: toggle,
                accessory: if unread > 0 { Accessory::Badge(unread) } else { more(path) },
                state: RowState { selection: Selection::of(&current(), &path), drop: drop_on(over(), path), ..RowState::default() },
                common: place(path),
                onclick: move |_| current.set(path),
                onpointerenter: move |_| over.set(Some(path)),
                onpointerleave: move |_| over.set(None),
                onpointerup: move |_| said.set(format!("dropped on {path}")),
                {children}
            }
        }
    };
    rsx! {
        Section {
            title: "Row: a folder tree as an outline",
            note: "A branch row opens the rows under it with a triangle that turns over --t-quick and a body that opens over --t-move; the triangle's press toggles and the row's selects. A leaf keeps the triangle's space. During a drag every folder is DropState::Accepts and the one under the pointer Target.",
            div { class: "g-side",
                List::<&'static str> {
                    label: "Folders",
                    style: ListStyle::SourceList,
                    items: vec![
                        ListItem::row("Projects", "Projects", branch(
                            "INBOX/Projects", "Projects", Icon::Folder, projects(),
                            EventHandler::new(move |to| projects.set(to)), 4,
                            rsx! {
                                {leaf("INBOX/Projects/Quire", "Quire")}
                                {leaf("INBOX/Projects/Sill", "Sill")}
                            },
                        )),
                        ListItem::row("Archive", "Archive", branch(
                            "Archive", "Archive", Icon::Archive, archive(),
                            EventHandler::new(move |to| archive.set(to)), 0,
                            rsx! { {leaf("Archive/2025", "2025")} },
                        )),
                        ListItem::row("Receipts", "Receipts", leaf("Receipts", "Receipts")),
                    ],
                }
            }
            p { class: "g-note", "Last: {said}" }
        }
    }
}
