//! A person on a draft or an event: their avatar, name and address.

use crate::components::companion::draft::model::PersonLine;
use crate::components::content::avatar::Avatar;
use dioxus::prelude::*;

/// One person as a line.
#[component]
pub fn PersonView(person: PersonLine) -> Element {
    let face = person.avatar;
    rsx! {
        span { class: "ds-card-person",
            Avatar { initial: face.initial, size: face.size, tone: face.tone, shape: face.shape }
            span { class: "ds-card-person-name", "{person.name}" }
            span { class: "ds-card-person-address", "{person.address}" }
        }
    }
}
