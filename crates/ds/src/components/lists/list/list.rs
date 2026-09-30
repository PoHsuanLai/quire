//! List: the container rows live in (`NSTableView` / `NSOutlineView`, design/30 section 2.6).
//! Items come and go by the roster (an inserted row fades and slides in, a removed one leaves
//! and the rows below close the gap), the arrow keys, Home and End rove among the rows and stop
//! at the ends, letters jump to the next label that starts with them, and the list draws its
//! selection accent while it holds the keyboard and grey when it does not.

use crate::components::lists::list::entry::ListEntry;
use crate::components::lists::list::keys::{ListKey, labelled, list_key, moved};
use crate::components::lists::list::model::{ListItem, ListStyle};
use crate::root::common::Common;
use crate::stack::typeahead::Typeahead;
use dioxus::prelude::*;
use ds_core::time::clock::now;
use ds_core::word::Word;
use ds_motion::presence::Exit;
use ds_motion::roster::RowPitch;
use ds_motion::use_roster::{LeaveBy, RosterSpec, use_roster};
use ds_style::tokens::control_size::SidebarSize;
use std::hash::Hash;

/// Whether the list holds the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Held {
    Yes,
    No,
}

/// A list of `items`, the whole list on every render: a key missing from it leaves. `cursor` is
/// the row the keys rest on (the caller draws it selected through the row's own state);
/// `onselect` hears where an arrow, Home, End or a typed letter moved it, and `onpick` a press
/// of Enter or Space on it. Rows hear their own clicks.
///
/// The keys that go missing in one render are one batch and leave together, then the rows below
/// heal by the heights the leaving rows measured. `exit` is read when a batch starts;
/// `on_settled` hears each dropped key once its batch has settled. Multi-selection is the
/// caller's: it puts each selected row's own state, and the list only moves the cursor. Keys are
/// hashed to name the items' nodes, so a key must hash the same on every render.
#[component]
pub fn List<K: Clone + PartialEq + Hash + 'static>(
    label: String,
    items: Vec<ListItem<K>>,
    #[props(default)] style: ListStyle,
    #[props(default)] sidebar: SidebarSize,
    #[props(default)] cursor: Option<K>,
    #[props(default)] onselect: Option<EventHandler<K>>,
    #[props(default)] onpick: Option<EventHandler<K>>,
    #[props(default = Exit::Row)] exit: Exit,
    #[props(default)] on_settled: Option<EventHandler<K>>,
    #[props(default)] common: Common,
) -> Element {
    let book = use_book(&items);
    let keys: Vec<K> = items.iter().map(|item| item.key.clone()).collect();
    let roster = use_roster(
        keys,
        RosterSpec {
            leave: LeaveBy::Delist,
            exit,
            pitch: RowPitch::default(),
            on_settled: Some(EventHandler::new(move |key: K| {
                book.forget(&key);
                if let Some(on_settled) = on_settled {
                    on_settled.call(key);
                }
            })),
        },
    );
    let pitches = roster.pitches();
    let mut held = use_signal(|| Held::No);
    let typeahead = use_hook(|| CopyValue::new(Typeahead::default()));
    let data = common.data_attributes();
    let mounted = common.clone();
    let listed = items.clone();
    let cursor_key = cursor.clone();
    let onkey = move |event: KeyboardEvent| {
        let Some(act) = list_key(&event.key(), event.modifiers()) else {
            return;
        };
        match act {
            ListKey::Move(rove) => {
                if let (Some(key), Some(onselect)) =
                    (moved(&listed, cursor_key.as_ref(), rove), onselect)
                {
                    event.prevent_default();
                    onselect.call(key);
                }
            }
            ListKey::Pick => {
                if let (Some(key), Some(onpick)) = (cursor_key.clone(), onpick) {
                    event.prevent_default();
                    onpick.call(key);
                }
            }
            ListKey::Jump(text) => {
                let mut buffer = typeahead;
                let found = labelled(&listed, cursor_key.as_ref(), |labels, from| {
                    let (next, found) = buffer.peek().clone().typed(&text, now(), labels, from);
                    buffer.set(next);
                    found
                });
                if let (Some(key), Some(onselect)) = (found, onselect) {
                    onselect.call(key);
                }
            }
        }
    };
    let away = (held() == Held::No).then_some("away");
    rsx! {
        div {
            class: common.class("ds-list"),
            id: common.id.clone(),
            role: "listbox",
            tabindex: "0",
            "aria-label": common.aria_label.clone().unwrap_or(label),
            "data-style": style.slug(),
            "data-sidebar-size": (style == ListStyle::SourceList).then(|| sidebar.slug()),
            "data-focus": away,
            onmounted: move |event| mounted.mounted(event),
            onfocus: move |_| held.set(Held::Yes),
            onblur: move |_| held.set(Held::No),
            onkeydown: onkey,
            ..data,
            for entry in roster.entries() {
                ListEntry::<K> {
                    key: "{node_key(&entry.key)}",
                    entry: entry.clone(),
                    pitches,
                    content: book.of(&entry.key),
                }
            }
        }
    }
}

/// An item's node key: its key's hash, in hex.
fn node_key<K: Hash>(key: &K) -> String {
    use std::hash::{BuildHasher, BuildHasherDefault, DefaultHasher};
    format!(
        "{:016x}",
        BuildHasherDefault::<DefaultHasher>::default().hash_one(key)
    )
}

/// The last content each key was listed with, so an item the caller has dropped still draws while
/// it leaves.
struct Book<K: 'static>(CopyValue<Vec<(K, Element)>>);

impl<K: 'static> Clone for Book<K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K: 'static> Copy for Book<K> {}

impl<K: Clone + PartialEq + 'static> Book<K> {
    fn of(&self, key: &K) -> Element {
        self.0
            .peek()
            .iter()
            .find(|(held, _)| held == key)
            .map_or_else(|| rsx! {}, |(_, content)| content.clone())
    }

    fn forget(&self, key: &K) {
        let mut book = self.0;
        let _ = book
            .try_write()
            .map(|mut book| book.retain(|(held, _)| held != key));
    }
}

/// The book, refreshed from this render's items (a listed item is always its newest content; a
/// dropped one keeps its last).
fn use_book<K: Clone + PartialEq + 'static>(items: &[ListItem<K>]) -> Book<K> {
    let mut book = use_hook(|| CopyValue::new(Vec::<(K, Element)>::new()));
    let kept: Vec<(K, Element)> = book
        .peek()
        .iter()
        .filter(|(key, _)| items.iter().all(|item| item.key != *key))
        .cloned()
        .collect();
    let next = items
        .iter()
        .map(|item| (item.key.clone(), item.content.clone()))
        .chain(kept)
        .collect();
    book.set(next);
    Book(book)
}

#[cfg(test)]
mod tests {
    use super::node_key;

    #[test]
    fn a_key_names_the_same_node_every_time() {
        assert_eq!(node_key(&7u32), node_key(&7u32));
        assert_ne!(node_key(&7u32), node_key(&8u32));
        assert_eq!(node_key(&7u32).len(), 16);
    }
}
