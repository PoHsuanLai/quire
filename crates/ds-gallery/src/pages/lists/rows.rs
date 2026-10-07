//! Row: every leading element, every accessory, both heights and every state a row can be in
//! (design/30 section 2.6), inside `List`s so the selection draws as a list draws it.

use crate::pages::{Section, Specimen};
use dioxus::prelude::*;
use ds::components::content::avatar::AvatarFace;
use ds::components::content::avatar::AvatarShape;
use ds::components::content::avatar::AvatarSize;
use ds::components::content::avatar::AvatarTone;
use ds::components::content::avatar::PersonHue;
use ds::components::content::status::battery_state::{BatteryPower, BatteryState};
use ds::components::lists::list::model::ListStyle;
use ds::components::lists::row::shape::{ClipBody, RowShape};
use ds::components::lists::row::size::RowSize;
use ds::prelude::*;
use ds_core::vocab::RowState;

/// One row of a specimen list: `key`, its `Row`.
fn item(key: &'static str, row: Element) -> ListItem<&'static str> {
    ListItem::row(key, key, row)
}

/// The rows of the page.
#[component]
pub fn RowGallery() -> Element {
    rsx! {
        Accessories {}
        Leadings {}
        States {}
        Shapes {}
        crate::pages::lists::row_actions::RowActions {}
        Rename {}
    }
}

/// A row renamed in place: its `edit` slot holds a plain `TextField`.
#[component]
fn Rename() -> Element {
    let mut name = use_signal(|| "Receipts".to_owned());
    rsx! {
        Section {
            title: "Row: rename in place",
            note: "edit: a field where the words stood, in the row's own face; the leading and the accessory stay, and the keys and presses typed in it stay the field's (the list does not move its cursor, jump or pick).",
            div { class: "g-list g-stage-pad", style: "width:320px",
                List::<&'static str> {
                    label: "Folders",
                    style: ListStyle::SourceList,
                    cursor: Some("Receipts"),
                    items: vec![
                        item("Inbox", rsx! { Row { title: "Inbox", leading: RowLeading::Icon(Icon::Inbox), onclick: |_| {} } }),
                        item("Receipts", rsx! {
                            Row {
                                title: "Receipts",
                                leading: RowLeading::Icon(Icon::Folder),
                                state: RowState { selection: Selection::Selected, ..RowState::default() },
                                edit: rsx! {
                                    TextField { bezel: FieldBezel::Plain, label: "Rename folder", value: name(), oninput: move |next| name.set(next) }
                                },
                                onclick: |_| {},
                            }
                        }),
                    ],
                }
            }
        }
    }
}

/// Every accessory, in a settings-height row.
#[component]
fn Accessories() -> Element {
    let mut on = use_signal(|| Check::On);
    let battery = BatteryState {
        level: Fraction(840),
        power: BatteryPower::Battery,
        ..BatteryState::default()
    };
    let row = |title: &'static str, accessory: Accessory| {
        item(
            title,
            rsx! {
                Row { title, size: RowSize::Settings, accessory, onclick: |_| {} }
            },
        )
    };
    rsx! {
        Section {
            title: "Row: accessories",
            note: "The trailing end: Check (a dash while Mixed), a Toggle of its own (flipping it does not run the row), Chevron, Text, Glyph, Battery (the glyph and its percentage), Spinner, Badge (a count as plain semibold tabular text in the secondary ink, as Mail's sidebar draws it: no capsule), and a caller's Slot. A busy row shows the small spinner in place of any of them.",
            div { class: "g-list g-stage-pad",
                List::<&'static str> {
                    label: "Accessories",
                    style: ListStyle::Grouped,
                    items: vec![
                        row("None", Accessory::None),
                        row("Check on", Accessory::Check(Check::On)),
                        row("Check mixed", Accessory::Check(Check::Mixed)),
                        row("Check off", Accessory::Check(Check::Off)),
                        row("Toggle", Accessory::Toggle { value: on(), on_toggle: EventHandler::new(move |value| on.set(value)) }),
                        row("Chevron", Accessory::Chevron),
                        row("Text", Accessory::Text("Connected".to_string())),
                        row("Glyph", Accessory::Glyph(Icon::Lock)),
                        row("Battery", Accessory::Battery(battery)),
                        row("Spinner", Accessory::Spinner),
                        row("Badge", Accessory::Badge(12)),
                    ],
                }
            }
        }
    }
}

/// Every leading element and both heights.
#[component]
fn Leadings() -> Element {
    let person = AvatarFace {
        initial: 'D',
        size: AvatarSize::Size18,
        tone: AvatarTone::Person(PersonHue::of("Dana")),
        shape: AvatarShape::Round,
    };
    let row = |title: &'static str, size: RowSize, leading: RowLeading| {
        item(
            title,
            rsx! {
                Row { title, detail: TextLine::from("A second line in the faint ink"), size, leading, accessory: Accessory::Chevron }
            },
        )
    };
    rsx! {
        Section {
            title: "Row: leading and height",
            note: "A bare glyph, a glyph on a disc (the accent disc marks the item in use), a source, a letter, an avatar; 24 tall (Compact) or 44 (Settings).",
            div { class: "g-row g-row-top",
                Specimen { name: "Compact",
                    div { class: "g-list g-stage-pad", style: "width:300px",
                        List::<&'static str> {
                            label: "Compact",
                            items: vec![
                                row("Glyph", RowSize::Compact, RowLeading::Icon(Icon::Wifi)),
                                row("Disc off", RowSize::Compact, RowLeading::Disc(Icon::Wifi, Selection::Unselected)),
                                row("Disc on", RowSize::Compact, RowLeading::Disc(Icon::Wifi, Selection::Selected)),
                                row("Avatar", RowSize::Compact, RowLeading::Avatar(person)),
                                row("Letter", RowSize::Compact, RowLeading::Text("Q".to_string())),
                            ],
                        }
                    }
                }
                Specimen { name: "Settings",
                    div { class: "g-list g-stage-pad", style: "width:300px",
                        List::<&'static str> {
                            label: "Settings",
                            items: vec![
                                row("Glyph", RowSize::Settings, RowLeading::Icon(Icon::Wifi)),
                                row("Disc off", RowSize::Settings, RowLeading::Disc(Icon::Wifi, Selection::Unselected)),
                                row("Disc on", RowSize::Settings, RowLeading::Disc(Icon::Wifi, Selection::Selected)),
                                row("Avatar", RowSize::Settings, RowLeading::Avatar(person)),
                                row("Letter", RowSize::Settings, RowLeading::Text("Q".to_string())),
                            ],
                        }
                    }
                }
            }
        }
    }
}

/// The states: selected, unread, disabled, busy, and a row's part in a drag.
#[component]
fn States() -> Element {
    let mut at = use_signal(|| "Selected");
    let row = move |title: &'static str, state: RowState| {
        let state = RowState {
            selection: if at() == title {
                Selection::Selected
            } else {
                state.selection
            },
            ..state
        };
        item(
            title,
            rsx! {
                Row { title, state, leading: RowLeading::Icon(Icon::Inbox), onclick: move |_| at.set(title) }
            },
        )
    };
    let with = |patch: fn(RowState) -> RowState| patch(RowState::default());
    rsx! {
        Section {
            title: "Row: states",
            note: "Click a row (or use the arrows, Home, End and letters on the list): the selection is the accent under its ink while the list holds the keyboard and grey when it does not, and the whole highlight goes grey in an inactive window.",
            div { class: "g-list g-stage-pad", style: "width:320px",
                List::<&'static str> {
                    label: "States",
                    cursor: Some(at()),
                    onselect: move |key: &'static str| at.set(key),
                    items: vec![
                        row("Selected", RowState::default()),
                        row("Unread", with(|s| RowState { emphasis: Emphasis::Strong, ..s })),
                        row("Disabled", with(|s| RowState { availability: Availability::Disabled, ..s })),
                        row("Busy", with(|s| RowState { availability: Availability::Busy, ..s })),
                        row("Drop target", with(|s| RowState { drop: DropState::Target, ..s })),
                        row("Accepts a drop", with(|s| RowState { drop: DropState::Accepts, ..s })),
                        row("Dragged", with(|s| RowState { drop: DropState::Source, ..s })),
                    ],
                }
            }
        }
    }
}

/// The launcher's shapes, a chord and an action.
#[component]
fn Shapes() -> Element {
    let mut removed = use_signal(|| 0u32);
    let file = RowShape::File {
        thumb: None,
        location: "~/Documents".to_string(),
        modified: "Yesterday".to_string(),
    };
    let clip = RowShape::Clip {
        body: ClipBody::Text {
            excerpt: "fn main() {\n    println!(\"hello\");\n}".to_string(),
            lines: 3,
        },
        age: "2 min".to_string(),
    };
    let keys = Shortcut(vec![ShortcutKey::Super, ShortcutKey::Char('r')]);
    let action = RowAction::new(
        Icon::X,
        "Remove from recent",
        EventHandler::new(move |_| removed += 1),
    );
    rsx! {
        Section {
            title: "Row: shapes, chords and actions",
            note: "A file row leads with its thumbnail (a glyph here), names its folder under its name and ends with when it changed; a clipboard entry draws its text in the code face clipped to its lines. A chord shows on the selected row; an action acts without picking the row.",
            div { class: "g-list g-stage-pad", style: "width:420px",
                List::<&'static str> {
                    label: "Shapes",
                    cursor: Some("File"),
                    items: vec![
                        item("File", rsx! {
                            Row { title: "Quarterly report.pdf", leading: RowLeading::Icon(Icon::Folder), shape: file, state: RowState { selection: Selection::Selected, ..RowState::default() }, size: RowSize::Settings, chord: RowChord::on_selected(keys), accessory: Accessory::Text("PDF".to_string()) }
                        }),
                        item("Clip", rsx! {
                            Row { title: "Copied text", shape: clip, size: RowSize::Settings }
                        }),
                        item("Action", rsx! {
                            Row { title: "budget march", leading: RowLeading::Icon(Icon::Search), action, size: RowSize::Settings }
                        }),
                    ],
                }
            }
            p { class: "g-note", "Removed: {removed}" }
        }
    }
}
