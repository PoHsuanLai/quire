//! TextField: plain, secure and search fields, bezeled and plain, at each size, with help and a
//! rejected value, prefix and suffix, disabled and busy.

use crate::pages::{Section, Specimen};
use dioxus::prelude::*;
use ds::components::fields::text_field_model::{FieldRows, Invalid};
use ds::motion::detail::stamp::EventStamp;
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;

/// The TextField section.
#[component]
pub fn TextFieldSection() -> Element {
    let mut name = use_signal(|| "Dana Okafor".to_owned());
    let mut query = use_signal(|| "invoice".to_owned());
    let mut note =
        use_signal(|| "Thanks for the quick reply.\nI will send the files on Monday.".to_owned());
    let mut secret = use_signal(String::new);
    let rejected = Validity::Invalid(Invalid {
        message: TextLine::from("That address is not valid."),
        stamp: EventStamp(1),
    });
    rsx! {
        Section { title: "TextField", note: "NSTextField, NSSecureTextField and NSSearchField: the ring shows while the input has the caret; a secure field draws dots and keeps its text out of the markup; a search field has a magnifier and a clear button once there is text.",
            for size in ControlSize::ALL.iter().copied() {
                div { class: "g-row",
                    span { class: "g-name g-type-name", "{size.slug()}" }
                    div { style: "width:190px",
                        TextField { label: "Name", value: name(), size, placeholder: "Name", oninput: move |next| name.set(next) }
                    }
                    div { style: "width:190px",
                        TextField { label: "Empty", value: "", size, placeholder: "Add a person", oninput: |_| {} }
                    }
                    div { style: "width:190px",
                        TextField { label: "Search", value: query(), size, kind: FieldKind::Search, placeholder: "Search", oninput: move |next| query.set(next) }
                    }
                }
            }
            div { class: "g-row g-row-top",
                Specimen { name: "secure".to_owned(),
                    div { style: "width:190px",
                        TextField { label: "Password", value: "", kind: FieldKind::Secure, placeholder: "Password", oninput: move |next| secret.set(next) }
                    }
                }
                Specimen { name: "secure, rejected".to_owned(),
                    div { style: "width:190px",
                        TextField { label: "Password", value: "", kind: FieldKind::Secure, validity: rejected.clone(), oninput: |_| {} }
                    }
                }
                Specimen { name: "help".to_owned(),
                    div { style: "width:190px",
                        TextField { label: "Email", value: "dana@example.com", help: TextLine::from("We never share it."), oninput: |_| {} }
                    }
                }
                Specimen { name: "rejected".to_owned(),
                    div { style: "width:190px",
                        TextField { label: "Email", value: "dana@", validity: rejected, help: TextLine::from("We never share it."), oninput: |_| {} }
                    }
                }
            }
            div { class: "g-row g-row-top",
                Specimen { name: "prefix and suffix".to_owned(),
                    div { style: "width:190px",
                        TextField {
                            label: "Website",
                            value: "example",
                            prefix: rsx! { IconView { source: Icon::Globe.into() } },
                            suffix: rsx! { span { ".com" } },
                            oninput: |_| {},
                        }
                    }
                }
                Specimen { name: "search with tokens".to_owned(),
                    div { style: "width:250px",
                        TextField { label: "Search", value: "from:dana", kind: FieldKind::Search, tokens: vec!["from:dana".to_owned(), "has:file".to_owned()], oninput: |_| {} }
                    }
                }
                Specimen { name: "search, filled".to_owned(),
                    div { style: "width:250px",
                        TextField { label: "Search", value: "Dana", kind: FieldKind::Search, placeholder: "Search", oninput: |_| {} }
                    }
                }
                Specimen { name: "plain".to_owned(),
                    div { style: "width:190px; font-size:16px; font-weight:600",
                        TextField { label: "Title", value: "Renamed in place", bezel: FieldBezel::Plain, oninput: |_| {} }
                    }
                }
                Specimen { name: "multiline, three rows".to_owned(),
                    div { style: "width:250px",
                        TextField { label: "Note", value: note(), kind: FieldKind::Multiline, placeholder: "Add a note", oninput: move |next| note.set(next) }
                    }
                }
                Specimen { name: "multiline, empty, six rows".to_owned(),
                    div { style: "width:250px",
                        TextField { label: "Note", value: "", kind: FieldKind::Multiline, rows: FieldRows::Six, placeholder: "Add a note", oninput: |_| {} }
                    }
                }
                Specimen { name: "disabled".to_owned(),
                    div { style: "width:190px",
                        TextField { label: "Name", value: "Locked", availability: Availability::Disabled, oninput: |_| {} }
                    }
                }
                Specimen { name: "busy".to_owned(),
                    div { style: "width:190px",
                        TextField { label: "Name", value: "Checking", availability: Availability::Busy, oninput: |_| {} }
                    }
                }
            }
        }
    }
}
