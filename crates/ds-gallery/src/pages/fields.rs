//! The Controls page's fields: TextInput in every face and kind (text, password, secret,
//! file, multiline, bare), and the SearchField. Split from `controls.rs` to keep that page
//! under its size.

use super::{Section, Specimen};
use dioxus::prelude::*;
use ds::{
    Availability, FieldFace, Grow, InputVariant, Rows, SearchField, TextInput, TextInputKind,
};

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

/// mailo gaps 4's kinds and the bare face, each live.
#[component]
pub fn FieldKinds() -> Element {
    let mut heard = use_signal(|| 0usize);
    let mut committed = use_signal(|| 0usize);
    let mut picks = use_signal(|| 0u32);
    let mut notes = use_signal(|| "Poh Lai\nAcme".to_string());
    let mut fixed = use_signal(String::new);
    let mut title = use_signal(|| "Work".to_string());
    let chosen = if picks() == 0 {
        String::new()
    } else {
        format!("signature-{picks}.png")
    };
    rsx! {
        Section { title: "TextInput kinds: secret, file, multiline, bare", note: "Secret keeps its text out of the markup (only its length is shown here, as the callbacks hear it; Enter or leaving it commits). File asks the host to choose (no native picker on Blitz): each pick here names a new file. Multiline grows a row per line, or stays at its rows. Bare takes the face of the text it sits in.",
            div { class: "g-grid2",
                Specimen { name: "secret: heard {heard} characters, committed {committed}",
                    TextInput { variant: InputVariant::Boxed, kind: TextInputKind::Secret, label: "App password", value: "", placeholder: "App password",
                        oninput: move |text: String| heard.set(text.chars().count()),
                        onchange: move |text: String| committed.set(text.chars().count()) }
                }
                Specimen { name: "file: {picks} picks",
                    TextInput { variant: InputVariant::Boxed, kind: TextInputKind::File, label: "Signature image", value: chosen, placeholder: "No file chosen",
                        oninput: |_| {}, on_pick: move |()| picks += 1 }
                }
                Specimen { name: "multiline, grows from 2 rows",
                    TextInput { variant: InputVariant::Boxed, kind: TextInputKind::Multiline { rows: Rows(2), grow: Grow::ToContent }, label: "Signature", value: notes(), placeholder: "Your signature",
                        oninput: move |text| notes.set(text) }
                }
                Specimen { name: "multiline, fixed at 3 rows",
                    TextInput { variant: InputVariant::Boxed, kind: TextInputKind::Multiline { rows: Rows(3), grow: Grow::Fixed }, label: "Note", value: fixed(), placeholder: "A note",
                        oninput: move |text| fixed.set(text) }
                }
                Specimen { name: "bare, in a title",
                    h3 { class: "g-bare-title", TextInput { variant: FieldFace::Bare, label: "Space name", value: title(), placeholder: "Name this Space", oninput: move |text| title.set(text) } }
                }
                Specimen { name: "bare, in a property row",
                    div { class: "g-row",
                        span { class: "g-name", "Folder" }
                        TextInput { variant: FieldFace::Bare, label: "Folder", value: "Receipts/2026", oninput: |_| {} }
                    }
                }
            }
        }
    }
}
