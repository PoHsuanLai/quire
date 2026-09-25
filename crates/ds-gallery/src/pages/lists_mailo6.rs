//! The Lists page's mailo gaps 6 specimen: a folder tree of `TreeItem`s during a drag, each
//! folder's ⋯ carrying `data-folder`. Split from `lists.rs` to keep that page under its size.

use super::Section;
use dioxus::prelude::*;
use ds::{
    DataAttr, DataName, Disclosure, DropState, Here, Icon, IconButton, IconButtonVariant, PlaceId,
    Propagation, TreeItem, TreeShape,
};

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
pub fn FolderTree() -> Element {
    // Posed with Archive under the pointer, as a drag over it would leave it.
    let mut over = use_signal(|| Some("Archive"));
    let mut said = use_signal(|| "nothing yet".to_string());
    let mut current = use_signal(|| "INBOX/Projects/Quire");
    let mut projects = use_signal(|| Disclosure::Open);
    let mut archive = use_signal(|| Disclosure::Closed);
    let more = move |path: &'static str| {
        rsx! {
            IconButton { variant: IconButtonVariant::Strip, icon: Icon::Ellipsis, label: "Actions for {path}",
                data: folder(path), propagation: Propagation::Stop,
                onclick: move |_| said.set(format!("the menu for {path}")) }
        }
    };
    let here = move |path: &'static str| {
        if current() == path {
            Here::Current
        } else {
            Here::Elsewhere
        }
    };
    let leaf = move |path: &'static str, label: &'static str| {
        rsx! {
            TreeItem { label, open: Disclosure::Closed, on_toggle: |_| {}, shape: TreeShape::Leaf,
                glyph: Icon::Folder, here: here(path), drop: drop_on(over(), path), place: PlaceId(path.to_string()),
                onselect: move |_| current.set(path),
                onpointerenter: move |_| over.set(Some(path)),
                onpointerleave: move |_| over.set(None),
                onpointerup: move |_| said.set(format!("dropped on {path}")),
                trailing: more(path) }
        }
    };
    rsx! {
        Section {
            title: "TreeItem: a folder tree as drop places",
            note: "A details/summary row in the sidebar item's chrome. open is the app's (the chevron turns over --t-quick); the label selects without toggling; the ⋯ in the trailing slot carries data-folder and never toggles. During a drag every folder is DropState::Accepts and the one under the pointer Target, drawn by the same .ds-drop-place rules as SidebarItem.",
            div { class: "g-col g-stage-pad", style: "width:240px",
                TreeItem {
                    label: "Projects",
                    open: projects(),
                    on_toggle: move |to| projects.set(to),
                    glyph: Icon::Folder,
                    count: 4,
                    here: here("INBOX/Projects"),
                    drop: drop_on(over(), "INBOX/Projects"),
                    place: PlaceId("INBOX/Projects".to_string()),
                    onselect: move |_| current.set("INBOX/Projects"),
                    onpointerenter: move |_| over.set(Some("INBOX/Projects")),
                    onpointerleave: move |_| over.set(None),
                    onpointerup: move |_| said.set("dropped on INBOX/Projects".to_string()),
                    trailing: more("INBOX/Projects"),
                    {leaf("INBOX/Projects/Quire", "Quire")}
                    {leaf("INBOX/Projects/Sill", "Sill")}
                }
                TreeItem {
                    label: "Archive",
                    open: archive(),
                    on_toggle: move |to| archive.set(to),
                    glyph: Icon::Archive,
                    here: here("Archive"),
                    drop: drop_on(over(), "Archive"),
                    place: PlaceId("Archive".to_string()),
                    onselect: move |_| current.set("Archive"),
                    onpointerenter: move |_| over.set(Some("Archive")),
                    onpointerleave: move |_| over.set(None),
                    onpointerup: move |_| said.set("dropped on Archive".to_string()),
                    trailing: more("Archive"),
                    {leaf("Archive/2025", "2025")}
                }
                {leaf("Receipts", "Receipts")}
            }
            p { class: "g-note", "Last: {said}" }
        }
    }
}
