//! Matrix: one component, chosen here, in a cell for each scheme and accent, in the toolbar's
//! material.

use crate::axes::Axes;
use crate::pages::{Scope, Section};
use dioxus::prelude::*;
use ds::TextField;
use ds::Word;
use ds::{
    Accent, Accessory, Availability, Button, Check, Chip, ChipVariant, Emphasis, Fraction, Icon,
    LabelHue, List, ListItem, ListStyle, Material, Row, RowLeading, RowState, Scheme,
    SegmentedControl, Selection, Slider, Surface, ThreadRow, Toggle, Verdict,
};
use ds::{Answers, Bezel, ButtonRole, ControlSize};
use ds::{Choice, Tracking};

/// What the matrix can show.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Subject {
    Buttons,
    Chips,
    Switches,
    Row,
    Sidebar,
    Tabs,
    Field,
}

const SUBJECTS: [(Subject, &str); 7] = [
    (Subject::Buttons, "Buttons"),
    (Subject::Chips, "Chips"),
    (Subject::Switches, "Toggle and slider"),
    (Subject::Row, "List row"),
    (Subject::Sidebar, "Sidebar item"),
    (Subject::Tabs, "Tabs"),
    (Subject::Field, "Text input"),
];

/// The matrix page.
#[component]
pub fn MatrixPage() -> Element {
    let mut subject = use_signal(|| Subject::Buttons);
    let material = use_context::<Signal<Axes>>().read().material;
    let options = SUBJECTS
        .iter()
        .map(|(subject, name)| (*subject, name.to_string()))
        .collect::<Vec<_>>();
    rsx! {
        Section { title: "Component",
            SegmentedControl::<Subject> { label: "Component", choices: Choice::pairs(options), tracking: Tracking::SelectOne(subject()), size: ControlSize::Mini, onchange: move |next| subject.set(next) }
        }
        for scheme in Scheme::ALL.iter().copied() {
            Section { title: format!("{} scheme", scheme.slug()),
                div { class: "g-matrix",
                    for accent in Accent::ALL.iter().copied() {
                        div { class: "g-col",
                            span { class: "g-code", "{accent.label()}" }
                            Scope { scheme, accent, material: Material::Popover, frame: Some(ds::FrameTint::None),
                                Surface { material,
                                    div { class: "g-cell",
                                        Cell { subject: subject() }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The chosen component, in its common states.
#[component]
fn Cell(subject: Subject) -> Element {
    match subject {
        Subject::Buttons => rsx! {
            Button { answers: Answers::Return, label: "Send", onclick: |_| {} }
            Button { label: "Pressed", value: Some(Check::On), onclick: |_| {} }
            Button { bezel: Bezel::Inline, label: "Quiet", icon: Some(Icon::Archive), onclick: |_| {} }
            Button { role: ButtonRole::Destructive, size: ControlSize::Mini, label: "Off", availability: Availability::Disabled, onclick: |_| {} }
        },
        Subject::Chips => rsx! {
            Chip { variant: ChipVariant::Accent, text: "Accent" }
            Chip { variant: ChipVariant::Neutral, text: "Neutral" }
            Chip { variant: ChipVariant::Label(LabelHue::Green), text: "green" }
            Chip { variant: ChipVariant::Status(Verdict::Fail), text: "2.1 : 1" }
        },
        Subject::Switches => rsx! {
            Toggle { label: "On", value: Check::On, onchange: |_| {} }
            Toggle { label: "Off", value: Check::Off, onchange: |_| {} }
            Slider { label: "Level", value: Fraction(600), onchange: |_| {} }
        },
        Subject::Row => rsx! {
            List::<u8> {
                label: "Row",
                items: vec![ListItem::row(
                    0,
                    "Re: UIDL stability",
                    rsx! {
                        ThreadRow {
                            state: RowState { selection: Selection::Selected, emphasis: Emphasis::Strong, ..RowState::default() },
                            name: "Dana Okafor",
                            via: None,
                            subject: "Re: UIDL stability",
                            snippet: None,
                            time: "09:41",
                            tags: rsx! {},
                            star: None,
                            strip: None,
                            onclick: |_| {},
                        }
                    },
                )],
            }
        },
        Subject::Sidebar => rsx! {
            List::<u8> {
                label: "Sidebar",
                style: ListStyle::SourceList,
                cursor: Some(0),
                items: vec![
                    ListItem::row(0, "Inbox", rsx! {
                        Row { title: "Inbox", leading: RowLeading::Icon(Icon::Inbox), accessory: Accessory::Badge(12), state: RowState { selection: Selection::Selected, ..RowState::default() } }
                    }),
                    ListItem::row(1, "Starred", rsx! {
                        Row { title: "Starred", leading: RowLeading::Icon(Icon::Star), state: RowState { selection: Selection::Unselected, ..RowState::default() } }
                    }),
                ],
            }
        },
        Subject::Tabs => rsx! {
            SegmentedControl::<u8> { label: "Tabs", choices: Choice::pairs(vec![(0, "Inbox".to_string()), (1, "Sent".to_string())]), tracking: Tracking::SelectOne(0), onchange: |_| {} }
        },
        Subject::Field => rsx! {
            TextField { label: "Name", value: "", placeholder: "Your name", oninput: |_| {} }
            TextField { label: "Mail", value: "dana@example.org", oninput: |_| {} }
        },
    }
}
