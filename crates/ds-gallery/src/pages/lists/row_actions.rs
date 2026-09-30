//! A row's action with a menu, its overflow, and a question the row asks (design/30 section 2.6).

use crate::pages::{Section, Specimen};
use dioxus::prelude::*;
use ds::components::controls::button_model::ButtonRole;
use ds::components::lists::list::model::ListStyle;
use ds::components::lists::row::confirm::RowConfirm;
use ds::components::menus::pop_up_button::{PopUpButton, PopUpKind};
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;
use ds_core::vocab::RowState;

/// A folder's menu: what the row's ⋯ lists.
fn folder_menu() -> Vec<MenuItem<usize>> {
    vec![
        MenuItem::new(0, "Rename…"),
        MenuItem::new(1, "Mark all as read"),
        MenuItem::Separator,
        MenuItem::new(2, "Delete…"),
    ]
}

/// Folders whose ⋯ opens their menu; Delete makes the row ask.
#[component]
pub fn RowActions() -> Element {
    let mut said = use_signal(|| "nothing yet".to_string());
    let mut asking = use_signal(|| true);
    let more = rsx! {
        PopUpButton::<usize> {
            kind: PopUpKind::Overflow,
            size: ControlSize::Small,
            items: folder_menu(),
            onpick: move |value: usize| {
                said.set(format!("menu item {value}"));
                if value == 2 {
                    asking.set(true);
                }
            },
        }
    };
    let confirm = asking().then(|| RowConfirm {
        question: "Delete “Receipts” and its 12 messages?".to_string(),
        confirm: "Delete".to_string(),
        role: ButtonRole::Destructive,
        on_confirm: EventHandler::new(move |()| {
            said.set("deleted".to_string());
            asking.set(false);
        }),
        on_cancel: EventHandler::new(move |()| asking.set(false)),
    });
    let plain = rsx! {
        PopUpButton::<usize> {
            kind: PopUpKind::Overflow,
            size: ControlSize::Small,
            items: folder_menu(),
            onpick: |_| {},
        }
    };
    rsx! {
        Section {
            title: "Row: overflow and confirmation",
            note: "An overflow (a PopUpButton of PopUpKind::Overflow in the row's slot) is the row's ⋯, Finder's Action button: pressing it opens its menu under the button and a pick reaches onpick, the row never picked. A row can ask a question in its own line (RowConfirm): the words give way to it, Cancel takes the keyboard and Escape answers it, and only the confirming button confirms.",
            div { class: "g-row g-row-top",
                Specimen { name: "Overflow", code: "Accessory::Slot(PopUpButton { kind: PopUpKind::Overflow, .. })".to_string(),
                    div { class: "g-list g-stage-pad", style: "width:320px",
                        List::<&'static str> {
                            label: "Folders",
                            style: ListStyle::SourceList,
                            items: vec![ListItem::row("Inbox", "Inbox", rsx! {
                                Row { title: "Inbox", leading: RowLeading::Icon(Icon::Inbox), accessory: Accessory::Slot(plain) }
                            })],
                        }
                    }
                }
                Specimen { name: "Asking", code: "Row { confirm: Some(RowConfirm { .. }), .. }".to_string(),
                    div { class: "g-list g-stage-pad", style: "width:420px",
                        List::<&'static str> {
                            label: "Folder",
                            items: vec![ListItem::row("Receipts", "Receipts", rsx! {
                                Row {
                                    title: "Receipts",
                                    leading: RowLeading::Icon(Icon::Folder),
                                    accessory: Accessory::Slot(more),
                                    confirm,
                                    state: RowState::default(),
                                }
                            })],
                        }
                    }
                }
            }
            p { class: "g-note", "Said: {said}" }
        }
    }
}
