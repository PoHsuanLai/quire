//! The row cases: ThreadRow in every weight and star state, Row in every accessory, leading
//! element and state, and List, with the helpers that draw a thread row the way a consumer does.

use crate::cases::Case;
use crate::scoped::Scoped;
use dioxus::prelude::*;
use ds::components::app::hover_strip::{ActionId, HoverStrip, StripAction};
use ds::components::app::thread_row::ThreadRow;
use ds::components::content::provider_mark::{MarkProvider, MarkStyle};
use ds::components::content::status::battery_state::BatteryState;
use ds::components::controls::button_model::Bezel;
use ds::components::controls::button_model::ButtonRole;
use ds::components::controls::button_model::ImagePosition;
use ds::components::controls::chip::{Chip, ChipVariant};
use ds::components::lists::list::model::ListStyle;
use ds::components::lists::row::confirm::RowConfirm;
use ds::components::lists::row::motion::RowMotion;
use ds::components::lists::row::shape::{ClipBody, RowShape};
use ds::components::lists::row::size::RowSize;
use ds::prelude::*;
use ds::root::common::Common;
use ds::root::pass_through::DataAttr;
use ds::root::pass_through::DataName;
use ds_core::vocab::RowState;
use ds_style::tokens::control_size::ControlSize;
use ds_style::tokens::control_size::SidebarSize;

/// The four strip actions of the Spaces prototype (`S:1286-1288`).
pub fn strip_actions() -> Vec<StripAction> {
    [
        (
            "archive",
            Icon::Archive,
            "Archive",
            "Archive → out of Inbox",
        ),
        ("snooze", Icon::Clock, "Snooze", "Snooze until…"),
        ("label", Icon::Tag, "Label", "Label…"),
        ("read", Icon::MailOpen, "Mark read", "Mark read"),
    ]
    .into_iter()
    .map(|(id, icon, label, fly)| StripAction {
        id: ActionId(id.to_string()),
        icon,
        label: label.to_string(),
        fly: fly.to_string(),
        onhover: None,
        onclick: EventHandler::new(|_| {}),
    })
    .collect()
}

/// A thread row, unread or read, with the via, tags and strip filled.
pub fn thread(emphasis: Emphasis, selection: Selection, star: Check) -> Element {
    thread_in_drag(emphasis, selection, star, DropState::Idle)
}

/// A read thread row playing `drop` in a drag.
fn dragged_thread(drop: DropState) -> Element {
    thread_in_drag(Emphasis::Plain, Selection::Unselected, Check::Off, drop)
}

/// [`thread`], playing `drop` in a drag.
fn thread_in_drag(
    emphasis: Emphasis,
    selection: Selection,
    star: Check,
    drop: DropState,
) -> Element {
    rsx! {
        ThreadRow {
            state: RowState { selection, emphasis, drop, ..RowState::default() },
            name: "Dana Okafor",
            via: rsx! {
                ProviderMark { provider: MarkProvider::Google, size: ControlSize::Mini, style: MarkStyle::Letter }
                "gmail"
            },
            subject: "Re: UIDL stability across servers",
            snippet: "Treat UIDL as stable only while UIDVALIDITY holds.".to_string(),
            time: "09:41",
            tags: rsx! { Chip { variant: ChipVariant::Accent, text: "spec" } },
            star: (star, EventHandler::new(|_| {})),
            strip: rsx! { HoverStrip { actions: strip_actions() } },
            onclick: |_| {},
        }
    }
}

/// A read thread row in a list: what a roster draws per entry, keyed by the consumer.
pub fn listed_thread(key: &'static str, emphasis: Emphasis) -> ListItem<&'static str> {
    ListItem::row(
        key,
        key,
        thread(emphasis, Selection::Unselected, Check::Off),
    )
}

/// A row named `title` with `accessory`, at settings height.
fn with_accessory(title: &'static str, accessory: Accessory) -> Element {
    rsx! { Scoped { Row { title, size: RowSize::Settings, accessory } } }
}

/// A row in `state`.
fn in_state(state: RowState) -> Element {
    rsx! { Scoped { Row { title: "Inbox", leading: RowLeading::Icon(Icon::Inbox), state } } }
}

/// A place named for a drag, playing `drop`.
fn place(drop: DropState) -> Element {
    rsx! {
        Row {
            title: "Archive",
            leading: RowLeading::Icon(Icon::Archive),
            state: RowState { drop, ..RowState::default() },
            common: Common { data: place_name("archive"), ..Common::default() },
            onpointerenter: |_| {},
            onpointerleave: |_| {},
            onpointerup: |_| {},
        }
    }
}

/// `data-place="<name>"`.
fn place_name(name: &str) -> Vec<DataAttr> {
    match DataName::parse("place") {
        Ok(attribute) => vec![DataAttr::new(attribute, name)],
        Err(error) => panic!("{error}"),
    }
}

pub const ROW_CASES: &[Case] = &[
    // ThreadRow: weight, selection, star, and a drag.
    Case {
        component: "thread_row",
        state: "unread",
        make: || thread(Emphasis::Strong, Selection::Unselected, Check::Off),
    },
    Case {
        component: "thread_row",
        state: "read",
        make: || thread(Emphasis::Plain, Selection::Unselected, Check::Off),
    },
    Case {
        component: "thread_row",
        state: "selected",
        make: || thread(Emphasis::Plain, Selection::Selected, Check::Off),
    },
    Case {
        component: "thread_row",
        state: "starred-at-rest",
        make: || thread(Emphasis::Plain, Selection::Unselected, Check::On),
    },
    Case {
        component: "thread_row",
        state: "drag-source",
        make: || dragged_thread(DropState::Source),
    },
    Case {
        component: "thread_row",
        state: "drop-target",
        make: || dragged_thread(DropState::Target),
    },
    Case {
        component: "thread_row",
        state: "bare",
        make: || {
            rsx! {
                ThreadRow {
                    state: RowState { selection: Selection::Unselected, emphasis: Emphasis::Plain, ..RowState::default() },
                    name: "Sam Lindqvist",
                    via: None,
                    subject: "Notes from the sync review",
                    snippet: None,
                    time: "Tue",
                    tags: rsx! {},
                    star: None,
                    strip: None,
                    onclick: |_| {},
                }
            }
        },
    },
    // Gallery fix A: a name longer than the column's 26-character budget fades; "bare" above
    // is the one that fits and does not.
    Case {
        component: "thread_row",
        state: "name-overflowing",
        make: || {
            rsx! {
                ThreadRow {
                    state: RowState { selection: Selection::Unselected, emphasis: Emphasis::Plain, ..RowState::default() },
                    name: "Maximilian Alexander von Hohenberg-Wittelsbach",
                    via: None,
                    subject: "Notes from the sync review",
                    snippet: None,
                    time: "Tue",
                    tags: rsx! {},
                    star: None,
                    strip: None,
                    onclick: |_| {},
                }
            }
        },
    },
    // Row: each accessory.
    Case {
        component: "row",
        state: "accessory-none",
        make: || with_accessory("None", Accessory::None),
    },
    Case {
        component: "row",
        state: "accessory-check-on",
        make: || with_accessory("Home", Accessory::Check(Check::On)),
    },
    Case {
        component: "row",
        state: "accessory-check-mixed",
        make: || with_accessory("Some", Accessory::Check(Check::Mixed)),
    },
    Case {
        component: "row",
        state: "accessory-check-off",
        make: || with_accessory("Café", Accessory::Check(Check::Off)),
    },
    Case {
        component: "row",
        state: "accessory-toggle",
        make: || {
            with_accessory(
                "Headphones",
                Accessory::Toggle {
                    value: Check::On,
                    on_toggle: EventHandler::new(|_| {}),
                },
            )
        },
    },
    Case {
        component: "row",
        state: "accessory-chevron",
        make: || with_accessory("Mouse", Accessory::Chevron),
    },
    Case {
        component: "row",
        state: "accessory-text",
        make: || with_accessory("Phone", Accessory::Text("Paired".to_string())),
    },
    Case {
        component: "row",
        state: "accessory-glyph",
        make: || with_accessory("Studio 5G", Accessory::Glyph(Icon::Lock)),
    },
    Case {
        component: "row",
        state: "accessory-battery",
        make: || {
            with_accessory(
                "Headphones",
                Accessory::Battery(BatteryState {
                    level: Fraction(840),
                    ..BatteryState::default()
                }),
            )
        },
    },
    Case {
        component: "row",
        state: "accessory-spinner",
        make: || with_accessory("Café", Accessory::Spinner),
    },
    Case {
        component: "row",
        state: "accessory-badge",
        make: || with_accessory("Inbox", Accessory::Badge(12)),
    },
    Case {
        component: "row",
        state: "accessory-slot",
        make: || {
            with_accessory(
                "Projects",
                Accessory::Slot(
                    rsx! { Button { bezel: Bezel::Toolbar, image: ImagePosition::Only, icon: Icon::Ellipsis, label: "Actions", onclick: |_| {} } },
                ),
            )
        },
    },
    // Row: leading elements and heights.
    Case {
        component: "row",
        state: "leading-icon",
        make: || rsx! { Row { title: "Wi-Fi", leading: RowLeading::Icon(Icon::Wifi) } },
    },
    Case {
        component: "row",
        state: "leading-disc-on",
        make: || rsx! { Row { title: "Home", leading: RowLeading::Disc(Icon::Wifi, Selection::Selected), size: RowSize::Settings } },
    },
    Case {
        component: "row",
        state: "leading-disc-off",
        make: || rsx! { Row { title: "Café", leading: RowLeading::Disc(Icon::Wifi, Selection::Unselected), size: RowSize::Settings } },
    },
    Case {
        component: "row",
        state: "leading-text",
        make: || rsx! { Row { title: "Quire", leading: RowLeading::Text("Q".to_string()), size: RowSize::Settings } },
    },
    Case {
        component: "row",
        state: "settings-detail",
        make: || rsx! { Row { title: "Home", detail: TextLine::from("Connected"), leading: RowLeading::Icon(Icon::Wifi), size: RowSize::Settings } },
    },
    // Row: states.
    Case {
        component: "row",
        state: "selected",
        make: || {
            in_state(RowState {
                selection: Selection::Selected,
                ..RowState::default()
            })
        },
    },
    Case {
        component: "row",
        state: "unread",
        make: || {
            in_state(RowState {
                emphasis: Emphasis::Strong,
                ..RowState::default()
            })
        },
    },
    Case {
        component: "row",
        state: "disabled",
        make: || {
            in_state(RowState {
                availability: Availability::Disabled,
                ..RowState::default()
            })
        },
    },
    Case {
        component: "row",
        state: "busy",
        make: || {
            in_state(RowState {
                availability: Availability::Busy,
                ..RowState::default()
            })
        },
    },
    Case {
        component: "row",
        state: "drag-source",
        make: || {
            in_state(RowState {
                drop: DropState::Source,
                ..RowState::default()
            })
        },
    },
    Case {
        component: "row",
        state: "drop-target",
        make: || {
            in_state(RowState {
                drop: DropState::Target,
                ..RowState::default()
            })
        },
    },
    Case {
        component: "row",
        state: "drop-accepts",
        make: || {
            in_state(RowState {
                drop: DropState::Accepts,
                ..RowState::default()
            })
        },
    },
    Case {
        component: "row",
        state: "place-named",
        make: || place(DropState::Idle),
    },
    Case {
        component: "row",
        state: "place-drop-target",
        make: || place(DropState::Target),
    },
    Case {
        component: "row",
        state: "marked-title",
        make: || rsx! { Row { title: "Uidl notes", marks: vec![0, 1, 2, 3] } },
    },
    Case {
        component: "row",
        state: "motion-in",
        make: || rsx! { Row { title: "Added", motion: RowMotion::In } },
    },
    Case {
        component: "row",
        state: "chord-selected",
        make: || rsx! { Row { title: "Invoice.pdf", state: RowState { selection: Selection::Selected, ..RowState::default() }, chord: RowChord::on_selected(Shortcut(vec![ShortcutKey::Super, ShortcutKey::Char('r')])) } },
    },
    Case {
        component: "row",
        state: "shape-file",
        make: || rsx! { Row { title: "Invoice.pdf", shape: RowShape::File { thumb: None, location: "~/Documents".to_string(), modified: "Yesterday".to_string() }, size: RowSize::Settings } },
    },
    Case {
        component: "row",
        state: "shape-clip-text",
        make: || rsx! { Row { title: "Copied", shape: RowShape::Clip { body: ClipBody::Text { excerpt: "fn main() {}".to_string(), lines: 2 }, age: "2 min".to_string() } } },
    },
    Case {
        component: "row",
        state: "action",
        make: || rsx! { Row { title: "from:dana", action: RowAction::new(Icon::X, "Remove from recent", EventHandler::new(|_| {})) } },
    },
    // A row's action with the consumer's own handle on it; the row asking.
    Case {
        component: "row",
        state: "action-common",
        make: || rsx! { Row { title: "Receipts", action: RowAction::new(Icon::Ellipsis, "More", EventHandler::new(|_| {})).with_common(Common { id: Some("receipts-more".to_string()), ..Common::default() }) } },
    },
    Case {
        component: "row",
        state: "confirm",
        make: || rsx! { Row { title: "Receipts", accessory: Accessory::Badge(12), confirm: Some(RowConfirm { question: "Delete “Receipts” and its 12 messages?".to_string(), confirm: "Delete".to_string(), role: ButtonRole::Destructive, on_confirm: EventHandler::new(|()| {}), on_cancel: EventHandler::new(|()| {}) }) } },
    },
    // List: each style, a heading among rows, and an item's exit.
    Case {
        component: "list",
        state: "plain",
        make: || rsx! { Scoped { List::<&'static str> { label: "Threads", items: vec![listed_thread("a", Emphasis::Strong)] } } },
    },
    Case {
        component: "list",
        state: "inset",
        make: || rsx! { Scoped { List::<&'static str> { label: "Networks", style: ListStyle::Inset, items: vec![ListItem::row("Home", "Home", with_accessory("Home", Accessory::Check(Check::On))), ListItem::row("Café", "Café", with_accessory("Café", Accessory::Chevron))] } } },
    },
    Case {
        component: "list",
        state: "source-list",
        make: || rsx! { Scoped { List::<&'static str> { label: "Places", style: ListStyle::SourceList, sidebar: SidebarSize::Large, items: vec![ListItem::heading("Favourites", rsx! { SectionHeader { title: "Favourites" } }), ListItem::row("Inbox", "Inbox", in_state(RowState::default()))] } } },
    },
    Case {
        component: "list",
        state: "cursor-held",
        make: || rsx! { Scoped { List::<&'static str> { label: "Places", cursor: Some("Inbox"), items: vec![ListItem::row("Inbox", "Inbox", in_state(RowState { selection: Selection::Selected, ..RowState::default() }))] } } },
    },
];
