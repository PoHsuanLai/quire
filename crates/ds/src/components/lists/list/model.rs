//! What a `List` is given: its style and its items (data only).

use dioxus::prelude::*;
use ds_core::vocab::Availability;
use ds_core::word::Word;

/// How a list is drawn (`NSTableView` styles, design/30 section 2.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum ListStyle {
    /// Rows on the surface, edge to edge.
    #[default]
    Plain,
    /// Rows in an inset card, a hairline between them (a settings pane).
    Inset,
    /// A sidebar's list: rows at the sidebar's size, headers that collapse.
    SourceList,
}

/// What an item is to the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ListRole {
    /// A row: a stop of the arrow keys and of type-to-select.
    #[default]
    Row,
    /// A header or a separator: drawn in its place, passed over by the keys.
    Heading,
}

/// One item of a list: its identity, what the keyboard needs to know of it, and what it draws.
#[derive(Debug, Clone, PartialEq)]
pub struct ListItem<K> {
    /// The item's identity, stable across renders (a thread id, a network's name). An item that
    /// stops being listed plays its exit and stays drawn until it settles.
    pub key: K,
    /// What type-to-select matches, and what a screen reader reads.
    pub label: String,
    /// Whether the keys may rest on it.
    pub availability: Availability,
    /// Whether it is a row or a heading.
    pub role: ListRole,
    /// What it draws: a `Row`, or a `SectionHeader`.
    pub content: Element,
}

impl<K> ListItem<K> {
    /// A row `key`, named `label`, drawing `content`, enabled.
    pub fn row(key: K, label: impl Into<String>, content: Element) -> Self {
        ListItem {
            key,
            label: label.into(),
            availability: Availability::Enabled,
            role: ListRole::Row,
            content,
        }
    }

    /// A heading `key` drawing `content`.
    pub fn heading(key: K, content: Element) -> Self {
        ListItem {
            key,
            label: String::new(),
            availability: Availability::Disabled,
            role: ListRole::Heading,
            content,
        }
    }

    /// The same item with `availability`.
    pub fn with(self, availability: Availability) -> Self {
        ListItem {
            availability,
            ..self
        }
    }
}
