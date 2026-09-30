//! Toolbar: leading items, title and trailing items; a subtitle; a control in the title's place;
//! and the overflow chevron when the room is short.

use crate::pages::Section;
use dioxus::prelude::*;
use ds::{
    Availability, Check, Choice, Icon, Px, SegmentedControl, TextLine, Toolbar, ToolbarItem,
    ToolbarRoom, Tracking,
};

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

/// The Toolbar section.
#[component]
pub fn ToolbarSection() -> Element {
    let mut said = use_signal(|| "nothing yet".to_owned());
    let mut view = use_signal(|| 0u8);
    rsx! {
        Section { title: "Toolbar", note: "NSToolbar: a 52 px band. Items give way from the last trailing one when the room is short and the chevron opens them as a menu; a toggle shows its state; a control can stand in the title's place.",
            div { class: "g-list", style: "width:620px",
                Toolbar::<&'static str> {
                    leading: leading(),
                    trailing: trailing(),
                    title: Some(TextLine::from("Downloads")),
                    subtitle: Some(TextLine::from("14 items")),
                    room: ToolbarRoom::Fixed(Px(620.0)),
                    onpick: move |value: &'static str| said.set(value.to_owned()),
                }
            }
            div { class: "g-list", style: "width:330px",
                Toolbar::<&'static str> {
                    leading: leading(),
                    trailing: trailing(),
                    title: Some(TextLine::from("Downloads")),
                    room: ToolbarRoom::Fixed(Px(330.0)),
                    onpick: move |value: &'static str| said.set(value.to_owned()),
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
                    onpick: move |value: &'static str| said.set(value.to_owned()),
                }
            }
            p { class: "g-code", "last picked: {said}" }
        }
    }
}
