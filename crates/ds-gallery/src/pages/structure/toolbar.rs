//! Toolbar: leading items, title and trailing items; a subtitle; a control in the title's place;
//! the overflow chevron when the room is short; and a menu hung from the item that was picked.

use crate::pages::Section;
use dioxus::prelude::*;
use ds::components::chrome::toolbar::model::{Picked, ToolbarItem, ToolbarRoom};
use ds::components::chrome::toolbar::search::{SearchSeat, ToolbarSearch};
use ds::components::chrome::toolbar::view::Toolbar;
use ds::components::controls::segmented::Tracking;
use ds::host::measure::Anchor;
use ds::prelude::*;

/// The items either side of the title.
fn leading() -> Vec<ToolbarItem<&'static str>> {
    vec![
        ToolbarItem::new("back", "Back", Icon::ChevronLeft),
        ToolbarItem::new("forward", "Forward", Icon::ChevronRight).with(Availability::Disabled),
    ]
}

fn trailing() -> Vec<ToolbarItem<&'static str>> {
    vec![
        ToolbarItem::new("share", "Share", Icon::Upload),
        ToolbarItem::new("tag", "Tag", Icon::Tag).toggle(Check::On),
        ToolbarItem::new("search", "Search", Icon::Search),
    ]
}

/// A toolbar with a search item in `room`: a field while the band has room for it, a magnifier
/// button below that.
#[component]
fn SearchToolbar(room: f32) -> Element {
    let mut query = use_signal(String::new);
    let search = ToolbarSearch {
        value: query(),
        min_width: Px(180.0),
        field: Callback::new(move |seat: SearchSeat| {
            rsx! {
                SearchField::<&'static str> {
                    label: "Search",
                    value: query(),
                    placeholder: "Search",
                    oninput: move |next: String| query.set(next),
                    onpick: |_| {},
                    focus: seat.focus(),
                    onblur: seat.onblur(),
                }
            }
        }),
    };
    rsx! {
        div { class: "g-list", style: "width:{room}px",
            Toolbar::<&'static str> {
                leading: leading(),
                trailing: vec![ToolbarItem::new("share", "Share", Icon::Upload)],
                title: Some(TextLine::from("Downloads")),
                room: ToolbarRoom::Fixed(Px(room)),
                search: Some(search),
                onpick: |_: Picked<&'static str>| {},
            }
        }
    }
}

/// The Toolbar section.
#[component]
pub fn ToolbarSection() -> Element {
    let mut said = use_signal(|| "nothing yet".to_owned());
    let mut view = use_signal(|| 0u8);
    let mut hung = use_signal(|| None::<Anchor>);
    rsx! {
        Section { title: "Toolbar", note: "NSToolbar: a 52 px band. Items give way from the last trailing one when the room is short and the chevron opens them as a menu; a toggle shows its state; a control can stand in the title's place. A pick reports the item and its button (Picked), and Tag in the first toolbar hangs a menu from that button.",
            div { class: "g-list", style: "width:620px",
                Toolbar::<&'static str> {
                    leading: leading(),
                    trailing: trailing(),
                    title: Some(TextLine::from("Downloads")),
                    subtitle: Some(TextLine::from("14 items")),
                    room: ToolbarRoom::Fixed(Px(620.0)),
                    onpick: move |pick: Picked<&'static str>| match (pick.value, pick.anchor) {
                        ("tag", Some(anchor)) => hung.set(Some(anchor)),
                        (value, _) => said.set(value.to_owned()),
                    },
                }
            }
            div { class: "g-list", style: "width:330px",
                Toolbar::<&'static str> {
                    leading: leading(),
                    trailing: trailing(),
                    title: Some(TextLine::from("Downloads")),
                    room: ToolbarRoom::Fixed(Px(330.0)),
                    onpick: move |pick: Picked<&'static str>| said.set(pick.value.to_owned()),
                }
            }
            div { class: "g-list", style: "width:620px",
                Toolbar::<&'static str> {
                    trailing: trailing(),
                    center: rsx! {
                        SegmentedControl::<u8> {
                            label: "View",
                            choices: Choice::pairs([(0u8, "Icons"), (1, "List"), (2, "Columns")]),
                            tracking: Tracking::SelectOne(view()),
                            onchange: move |next| view.set(next),
                        }
                    },
                    room: ToolbarRoom::Fixed(Px(620.0)),
                    onpick: move |pick: Picked<&'static str>| said.set(pick.value.to_owned()),
                }
            }
            p { class: "g-note", "NSSearchToolbarItem: the search field keeps its minimum width while the band has room for it and its items; below that it collapses to a magnifier button, and pressing the button opens the field in place with the caret in it. An empty field folds back when the caret leaves it or on Escape." }
            SearchToolbar { room: 620.0 }
            SearchToolbar { room: 380.0 }
            p { class: "g-code", "last picked: {said}" }
            if let Some(anchor) = hung() {
                Menu::<&'static str> {
                    placement: MenuPlacement::Popup,
                    anchor,
                    items: vec![MenuItem::new("work", "Work"), MenuItem::new("travel", "Travel"), MenuItem::new("receipts", "Receipts")],
                    onpick: move |label: &'static str| {
                        said.set(format!("Tag: {label}"));
                        hung.set(None);
                    },
                    onclose: move |()| hung.set(None),
                }
            }
        }
    }
}
