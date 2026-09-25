//! Whether typing filters a menu's entries, and the query line a `Field` filter draws (mailo
//! gaps 5). Split from `menu_lines`.
//!
//! A `Typing` menu filters as keys arrive but shows nothing of what was typed, which suits a
//! short context menu where the matches say enough. A picker with many rows (labels, folders,
//! a From address) wants the query visible: `Field` draws it in a row at the top of the menu,
//! styled as an inline `TextInput`. The row is drawn, not a real input: the menu keeps the
//! keyboard (its keys are the same under either filter), so a field there would only take the
//! focus away from the cursor it drives.

use dioxus::prelude::*;

/// Whether typing filters the entries.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum Filter {
    /// Typing filters with the fuzzy ranker and resets the selection; no query line is drawn.
    Typing,
    /// As `Typing`, and the typed query is drawn in a field row at the top of the menu
    /// (`div.ds-menu-filter`), `placeholder` while nothing is typed. The row is not a choice:
    /// the cursor stays on the rows below it.
    Field {
        /// What the empty row says: "Filter labels…".
        placeholder: String,
    },
    /// The entries are fixed.
    #[default]
    None,
}

/// Whether keys type into the query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Typed {
    Yes,
    No,
}

impl Filter {
    /// Whether characters and Backspace edit the query.
    pub(crate) fn types(&self) -> Typed {
        match self {
            Filter::Typing | Filter::Field { .. } => Typed::Yes,
            Filter::None => Typed::No,
        }
    }
}

/// The query row a `Field` filter draws above the rows, or nothing. `role="searchbox"` names
/// it for assistive technology as a search field whose text is `typed`; the caret is a drawn
/// bar where the next character goes (before the placeholder, after the text), since the row itself never holds the focus.
pub(crate) fn filter_row(filter: &Filter, typed: &str) -> Element {
    let Filter::Field { placeholder } = filter else {
        return rsx! {};
    };
    rsx! {
        div {
            class: "ds-menu-filter",
            role: "searchbox",
            "aria-label": "{placeholder}",
            if typed.is_empty() {
                i { class: "ds-menu-filter-caret", "aria-hidden": "true" }
                span { class: "ds-menu-filter-placeholder", "{placeholder}" }
            } else {
                span { class: "ds-menu-filter-text", "{typed}" }
                i { class: "ds-menu-filter-caret", "aria-hidden": "true" }
            }
        }
    }
}
