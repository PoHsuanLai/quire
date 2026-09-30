//! Button: the bezels at each size, the states, the default and destructive buttons, the
//! image-only toolbar button, a toggle button and the busy one.

use crate::pages::{Section, Specimen};
use dioxus::prelude::*;
use ds::Word;
use ds::{
    Answers, Availability, Bezel, Button, ButtonRole, Check, ControlSize, Icon, ImagePosition,
    Shown,
};

/// The Button section.
#[component]
pub fn ButtonSection() -> Element {
    let mut on = use_signal(|| Check::Off);
    rsx! {
        Section { title: "Button", note: "NSButton: push, toolbar, inline and help bezels, each at the four sizes; press and hold one to see the pressed fill, Tab to one and press Return or Space.",
            for bezel in [Bezel::Push, Bezel::Toolbar, Bezel::Inline] {
                for size in ControlSize::ALL.iter().copied() {
                    div { class: "g-row",
                        span { class: "g-name g-type-name", "{bezel.slug()} {size.slug()}" }
                        Button { bezel, size, label: "Rest", onclick: |_| {} }
                        Button { bezel, size, label: "With icon", icon: Icon::Archive, onclick: |_| {} }
                        Button { bezel, size, label: "Disabled", availability: Availability::Disabled, onclick: |_| {} }
                        Button { bezel, size, label: "Busy", availability: Availability::Busy, onclick: |_| {} }
                        Button { bezel, size, label: "Open", icon: Icon::ChevronDown, shown: Shown::Visible, onclick: |_| {} }
                    }
                }
            }
            div { class: "g-row",
                span { class: "g-name g-type-name", "default, cancel, destructive" }
                Button { answers: Answers::Return, label: "Send", onclick: |_| {} }
                Button { answers: Answers::Escape, label: "Cancel", onclick: |_| {} }
                Button { role: ButtonRole::Destructive, label: "Delete", icon: Icon::Trash, onclick: |_| {} }
                Button { answers: Answers::Return, role: ButtonRole::Destructive, label: "Erase", onclick: |_| {} }
                Button { answers: Answers::Return, label: "Send", availability: Availability::Disabled, onclick: |_| {} }
                Button { answers: Answers::Return, label: "Sending", availability: Availability::Busy, onclick: |_| {} }
            }
            div { class: "g-row",
                span { class: "g-name g-type-name", "toggle button" }
                Button { label: "Bold", value: on(), onclick: move |_| on.set(on().flipped()) }
                Button { label: "On", value: Check::On, onclick: |_| {} }
                Button { label: "Off", value: Check::Off, onclick: |_| {} }
            }
            div { class: "g-row",
                span { class: "g-name g-type-name", "image only" }
                for size in ControlSize::ALL.iter().copied() {
                    Specimen { name: size.slug().to_owned(),
                        div { class: "g-row",
                            Button { bezel: Bezel::Toolbar, size, image: ImagePosition::Only, icon: Icon::Star, label: "Star", onclick: |_| {} }
                            Button { bezel: Bezel::Toolbar, size, image: ImagePosition::Only, icon: Icon::Star, label: "Starred", value: Check::On, onclick: |_| {} }
                            Button { bezel: Bezel::Toolbar, size, image: ImagePosition::Only, icon: Icon::Trash, label: "Delete", availability: Availability::Disabled, onclick: |_| {} }
                            Button { bezel: Bezel::Toolbar, size, image: ImagePosition::Only, icon: Icon::Refresh, label: "Refresh", availability: Availability::Busy, onclick: |_| {} }
                        }
                    }
                }
                Specimen { name: "help".to_owned(),
                    Button { bezel: Bezel::Help, label: "Help", onclick: |_| {} }
                }
            }
        }
    }
}
