//! The footer every card draws: which model served it and where, what it drew on, and the
//! context the prompt carried.

use super::footer::{CardFooter, ServedByView, ServedPlaceKind, SourceChip};
use crate::components::content::icon_view::IconView;
use dioxus::prelude::*;
use ds_core::word::Word;
use ds_intents::ContextChip;
use ds_style::icon::render::IconSize;

/// Where the model ran, in the card's words.
fn place_words(served: &ServedByView) -> String {
    match (served.place.kind, &served.place.provider) {
        (ServedPlaceKind::ThisComputer, _) => "on this computer".to_owned(),
        (ServedPlaceKind::LocalNetwork, _) => "on your network".to_owned(),
        (ServedPlaceKind::Cloud, Some(provider)) => format!("at {provider}"),
        (ServedPlaceKind::Cloud, None) => "in the cloud".to_owned(),
    }
}

/// A scope chip's words: its label, and the count where it stands for several.
fn scope_words(chip: &ContextChip) -> String {
    match chip.count {
        Some(count) => format!("{} {}", chip.label, count.0),
        None => chip.label.clone(),
    }
}

/// The footer: the served-by line, the sources, the scope.
#[component]
pub fn CardFooterView(footer: CardFooter) -> Element {
    let served = format!(
        "{}, {}",
        footer.served_by.model,
        place_words(&footer.served_by)
    );
    rsx! {
        div { class: "ds-card-footer",
            span {
                class: "ds-card-served",
                "data-place": footer.served_by.place.kind.slug(),
                "{served}"
            }
            for (index , source) in footer.sources.iter().enumerate() {
                SourceMark { key: "{index}", source: source.clone() }
            }
            for (index , chip) in footer.scope.iter().enumerate() {
                span { key: "{index}", class: "ds-card-scope", "data-kind": chip.kind.slug(), "{scope_words(chip)}" }
            }
        }
    }
}

#[component]
fn SourceMark(source: SourceChip) -> Element {
    rsx! {
        span { class: "ds-card-source",
            IconView { source: source.icon, size: IconSize::Compact }
            span { "{source.label}" }
        }
    }
}
