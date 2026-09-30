//! FieldRow and FieldGroup: System Settings rows in a group, a grid form (one control and several
//! that wrap) and a FactList.

use crate::pages::Section;
use dioxus::prelude::*;
use ds::components::content::text_runs::RunTone;
use ds::components::controls::checkbox::Checkbox;
use ds::components::controls::segmented::Tracking;
use ds::components::fields::fact_list::{Fact, FactList};
use ds::components::fields::field_row::{FieldGroup, FieldRow, RowLayout};
use ds::components::menus::pop_up_button::{PopUpButton, PopUpKind};
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;

/// The FieldRow section.
#[component]
pub fn FieldRowSection() -> Element {
    let mut wifi = use_signal(|| Check::On);
    let mut mode = use_signal(|| 1u8);
    let mut sign = use_signal(|| Check::On);
    let mut name = use_signal(|| "Dana's Laptop".to_owned());
    let mut language = use_signal(|| "en");
    let languages = || {
        vec![
            MenuItem::new("en", "English"),
            MenuItem::new("fr", "Français"),
            MenuItem::new("ja", "日本語"),
        ]
    };
    rsx! {
        Section { title: "FieldRow", note: "NSGridView form rows: the label column with its help line, the control column. A group is one inset card with a hairline between rows; a disabled row dims and takes no pointer. The form layout gives the labels a fixed column so the controls line up.",
            div { class: "g-row g-row-top",
                div { style: "width:440px",
                    FieldGroup { title: "Network",
                        FieldRow { label: TextLine::from("Wi-Fi"), help: Some(TextLine::from("Join known networks automatically.")),
                            Toggle { label: "Wi-Fi", value: wifi(), onchange: move |next| wifi.set(next) }
                        }
                        FieldRow { label: TextLine::from("Appearance"),
                            SegmentedControl::<u8> {
                                label: "Appearance",
                                choices: Choice::pairs([(0u8, "Light"), (1, "Dark"), (2, "Auto")]),
                                tracking: Tracking::SelectOne(mode()),
                                size: ControlSize::Small,
                                onchange: move |next| mode.set(next),
                            }
                        }
                        FieldRow { label: TextLine::from("Language"),
                            PopUpButton::<&'static str> {
                                items: languages(),
                                kind: PopUpKind::PopUp,
                                value: Some(language()),
                                onpick: move |next| language.set(next),
                            }
                        }
                        FieldRow { label: TextLine::from("Airplane mode"), availability: Availability::Disabled,
                            Toggle { label: "Airplane mode", value: Check::Off, availability: Availability::Disabled, onchange: |_| {} }
                        }
                    }
                }
                div { style: "width:400px",
                    FieldGroup { title: "About",
                        FieldRow { label: TextLine::from("Name"), layout: RowLayout::Form,
                            TextField { label: "Name", value: name(), oninput: move |next| name.set(next) }
                        }
                        FieldRow { label: TextLine::from("Model"), layout: RowLayout::Form,
                            span { "Framework 13" }
                        }
                        FieldRow { label: TextLine::from("Protection"), layout: RowLayout::Form,
                            Checkbox { label: "Sign", value: sign(), onchange: move |next| sign.set(next) }
                            Checkbox { label: "Encrypt", value: Check::Off, onchange: |_| {} }
                            Button { label: "More options", size: ControlSize::Small, onclick: |_| {} }
                        }
                    }
                }
                div { style: "width:400px",
                    FieldGroup { title: "Invitation, a FactList",
                        FactList { facts: vec![
                            Fact::new("When", "Tue 4 Nov, 10:00 to 11:00"),
                            Fact::new("Where", "Room 4"),
                            Fact::new("Who", TextLine::Runs(vec![
                                TextRun::new("Dana", RunTone::Strong),
                                TextRun::new(", Noor and 3 more", RunTone::Plain),
                            ])),
                        ] }
                    }
                }
            }
        }
    }
}
