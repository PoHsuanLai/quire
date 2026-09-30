//! FieldRow and FieldGroup: System Settings rows in a group, and a grid form.

use crate::pages::Section;
use dioxus::prelude::*;
use ds::{
    Availability, Check, Choice, ControlSize, FieldGroup, FieldRow, MenuItem, PopUpButton,
    PopUpKind, RowLayout, SegmentedControl, TextField, TextLine, Toggle, Tracking,
};

/// The FieldRow section.
#[component]
pub fn FieldRowSection() -> Element {
    let mut wifi = use_signal(|| Check::On);
    let mut mode = use_signal(|| 1u8);
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
                    }
                }
            }
        }
    }
}
