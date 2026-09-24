//! The Controls page's fields: TextInput in both variants and its password kind, and the
//! SearchField. Split from `controls.rs` to keep that page under its size.

use super::{Section, Specimen};
use dioxus::prelude::*;
use ds::{Availability, InputVariant, SearchField, TextInput, TextInputKind};

#[component]
pub fn Fields() -> Element {
    let mut text = use_signal(String::new);
    let mut search = use_signal(|| "invoice".to_string());
    let mut secret = use_signal(String::new);
    let mut typing = use_signal(|| "not in the field");
    rsx! {
        Section { title: "TextInput and SearchField", note: "Boxed and inline; empty with a placeholder, filled, disabled; a password, whose name follows its focus and blur. Type in the live ones.",
            div { class: "g-grid3",
                for variant in [InputVariant::Boxed, InputVariant::Inline] {
                    Specimen { name: "live",
                        TextInput { variant, label: "Live", value: text(), placeholder: "Type here", oninput: move |next| text.set(next) }
                    }
                    Specimen { name: "filled",
                        TextInput { variant, label: "Filled", value: "pohsuan@example.org", oninput: |_| {} }
                    }
                    Specimen { name: "disabled",
                        TextInput { variant, label: "Disabled", value: "", placeholder: "Not now", availability: Availability::Disabled, oninput: |_| {} }
                    }
                }
            }
            div { class: "g-grid2",
                Specimen { name: "password: {typing}",
                    TextInput { variant: InputVariant::Boxed, kind: TextInputKind::Password, label: "Password", value: secret(), placeholder: "App password",
                        oninput: move |next| secret.set(next), onfocus: move |()| typing.set("typing"), onblur: move |()| typing.set("not in the field") }
                }
                Specimen { name: "password, filled",
                    TextInput { variant: InputVariant::Boxed, kind: TextInputKind::Password, label: "Password", value: "hunter2", oninput: |_| {} }
                }
                Specimen { name: "search with tokens",
                    SearchField {
                        label: "Search",
                        value: search(),
                        placeholder: "Search mail",
                        tokens: vec!["from:dana".to_string(), "has:attachment".to_string()],
                        oninput: move |next| search.set(next),
                        onkey: |_| {},
                    }
                }
                Specimen { name: "search, empty",
                    SearchField { label: "Search", value: "", placeholder: "Search mail", tokens: Vec::new(), oninput: |_| {}, onkey: |_| {} }
                }
            }
        }
    }
}
