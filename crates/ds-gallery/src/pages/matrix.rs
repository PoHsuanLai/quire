//! Matrix: one component, chosen here, in a cell for each scheme and accent, in the toolbar's
//! material.

use super::{Scope, Section};
use crate::axes::Axes;
use dioxus::prelude::*;
use ds::{
    Accent, Anim, AnimatedList, Availability, Button, ButtonVariant, Chip, ChipVariant, Emphasis,
    Fraction, Here, Icon, InputVariant, ItemKind, LabelHue, ListPresence, ListRow, Material,
    Presence, PulseKey, Scheme, SegSize, SegmentedControl, Selection, SidebarItem, Slider,
    StaggerIndex, Surface, Switch, Tabs, TextInput, Toggle, Verdict,
};

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
            SegmentedControl::<Subject> { label: "Component", options, value: subject(), size: SegSize::Small, onchange: move |next| subject.set(next) }
        }
        for scheme in Scheme::ALL {
            Section { title: format!("{} scheme", scheme.slug()),
                div { class: "g-matrix",
                    for accent in Accent::ALL {
                        div { class: "g-col",
                            span { class: "g-code", "{accent.label()}" }
                            Scope { scheme, accent, material: Material::Popover,
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
            Button { variant: ButtonVariant::Primary, label: "Send", onclick: |_| {} }
            Button { variant: ButtonVariant::Secondary, label: "Pressed", pressed: Some(Switch::On), onclick: |_| {} }
            Button { variant: ButtonVariant::Quiet, label: "Quiet", icon: Some(Icon::Archive), onclick: |_| {} }
            Button { variant: ButtonVariant::Danger, label: "Off", availability: Availability::Disabled, onclick: |_| {} }
        },
        Subject::Chips => rsx! {
            Chip { variant: ChipVariant::Accent, text: "Accent" }
            Chip { variant: ChipVariant::Neutral, text: "Neutral" }
            Chip { variant: ChipVariant::Label(LabelHue::Green), text: "green" }
            Chip { variant: ChipVariant::Status(Verdict::Fail), text: "2.1 : 1" }
        },
        Subject::Switches => rsx! {
            Toggle { label: "On", value: Switch::On, onchange: |_| {} }
            Toggle { label: "Off", value: Switch::Off, onchange: |_| {} }
            Slider { label: "Level", value: Fraction(600), onchange: |_| {} }
        },
        Subject::Row => rsx! {
            AnimatedList { label: "Row", presence: ListPresence::Present,
                ListRow {
                    selection: Selection::Selected,
                    emphasis: Emphasis::Strong,
                    index: StaggerIndex::new(0),
                    presence: Presence::Present,
                    name: "Dana Okafor",
                    via: None,
                    subject: "Re: UIDL stability",
                    snippet: None,
                    time: "09:41",
                    tags: rsx! {},
                    star: None,
                    star_pulse: PulseKey::rest(Anim::StarPop),
                    strip: None,
                    onclick: |_| {},
                }
            }
        },
        Subject::Sidebar => rsx! {
            SidebarItem {
                kind: ItemKind::Place { icon: Icon::Inbox },
                label: "Inbox",
                here: Here::Current,
                count: Some(12),
                presence: Presence::Present,
                preview: None,
                pulse: PulseKey::rest(Anim::SealPop),
                onclick: |_| {},
                onclose: None,
            }
            SidebarItem {
                kind: ItemKind::Place { icon: Icon::Star },
                label: "Starred",
                here: Here::Elsewhere,
                count: None,
                presence: Presence::Present,
                preview: None,
                pulse: PulseKey::rest(Anim::SealPop),
                onclick: |_| {},
                onclose: None,
            }
        },
        Subject::Tabs => rsx! {
            Tabs::<u8> { label: "Tabs", tabs: vec![(0, "Inbox".to_string()), (1, "Sent".to_string())], value: 0, onchange: |_| {} }
        },
        Subject::Field => rsx! {
            TextInput { variant: InputVariant::Boxed, label: "Name", value: "", placeholder: "Your name", oninput: |_| {} }
            TextInput { variant: InputVariant::Boxed, label: "Mail", value: "dana@example.org", oninput: |_| {} }
        },
    }
}
