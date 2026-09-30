//! FactList: a read-only list of labels and their values (design/30 section 2.2): an invitation's
//! When, Where and Who; a certificate's issuer and dates. It is a form row without a control, so
//! it is drawn as `FieldRow`'s `Form` layout is: a fixed label column ending at the value, the
//! values lining up, and read as a description list by assistive technology.
//!
//! Markup: `dl.ds-fact-list` of `div.ds-fact-list-item` holding `dt.ds-fact-list-label` and
//! `dd.ds-fact-list-value`.

use crate::components::content::text_runs::{TextLine, text};
use crate::root::common::Common;
use dioxus::prelude::*;

/// One fact: what it is and what it says.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Fact {
    /// The name of the fact: "When".
    pub label: String,
    /// What it says, whole or as runs: "Tue 4 Nov, 10:00 to 11:00".
    pub value: TextLine,
}

impl Fact {
    /// The fact `label` with `value`.
    pub fn new(label: impl Into<String>, value: impl Into<TextLine>) -> Self {
        Fact {
            label: label.into(),
            value: value.into(),
        }
    }
}

/// The facts, one row each, in order. A fact whose value is empty still draws its label.
#[component]
pub fn FactList(facts: Vec<Fact>, #[props(default)] common: Common) -> Element {
    let data = common.data_attributes();
    rsx! {
        dl {
            id: common.id.clone(),
            class: common.class("ds-fact-list"),
            "aria-label": common.aria_label.clone(),
            onmounted: move |event| common.mounted(event),
            ..data,
            for (index , fact) in facts.into_iter().enumerate() {
                div { key: "{index}", class: "ds-fact-list-item",
                    dt { class: "ds-fact-list-label", "{fact.label}" }
                    dd { class: "ds-fact-list-value", {text(&fact.value)} }
                }
            }
        }
    }
}
