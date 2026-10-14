//! A draft reply: who it goes to, its subject, the message it answers and the body, which the
//! person edits in place.

use super::person::PersonView;
use crate::components::companion::draft::model::DraftReply;
use crate::components::companion::form::model::{FieldKeyId, FormValue};
use crate::components::fields::text_field::TextField;
use crate::components::fields::text_field_model::{FieldKind, FieldRows};
use dioxus::prelude::*;

/// The key an edit of the draft's body is reported under.
pub const BODY_KEY: &str = "body";

/// The draft as the card body; an edit of the body is reported under [`BODY_KEY`].
#[component]
pub fn DraftBody(draft: DraftReply, on_edit: EventHandler<(FieldKeyId, FormValue)>) -> Element {
    rsx! {
        div { class: "ds-card-draft",
            div { class: "ds-card-draft-to",
                span { class: "ds-card-label", "To" }
                for (index , person) in draft.to.iter().enumerate() {
                    PersonView { key: "{index}", person: person.clone() }
                }
            }
            div { class: "ds-card-draft-subject",
                span { class: "ds-card-label", "Subject" }
                span { "{draft.subject}" }
            }
            if let Some(quoted) = &draft.quoted {
                blockquote { class: "ds-card-quote", "{quoted}" }
            }
            TextField {
                label: "Reply",
                value: draft.body.clone(),
                kind: FieldKind::Multiline,
                rows: FieldRows::Six,
                oninput: move |value| on_edit.call((FieldKeyId(BODY_KEY.to_owned()), FormValue::Text(value))),
            }
        }
    }
}
