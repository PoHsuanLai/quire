//! LeavingList: a column of the consumer's rows that come and go with motion (the
//! notification center). The consumer lists its rows; a row it stops listing plays its exit
//! (`row-out` by default) and stays drawn until the exit settles; every row dropped in the same
//! render leaves as one batch; once the batch has settled the rows are dropped together and the
//! rows below heal by the heights the dropped rows measured, so each starts exactly where it
//! stood. A row that arrives plays `row-in`. Under Reduced motion a leaving row fades out
//! (`menu-out`), an arriving one fades in (`fade`), and the rows below move into place without
//! sliding.
//!
//! Each row is measured as it mounts and again as it starts to leave (its content may have
//! grown since), in a `div.ds-leaving-row` that contains its content's margins, so the measured
//! height is the whole distance the rows below it move.

use crate::components::lists::leaving_row::LeavingRow;
use crate::motion::presence::Exit;
use crate::motion::roster::RowPitch;
use crate::motion::use_roster::{LeaveBy, RosterSpec, use_roster};
use dioxus::prelude::*;
use std::hash::Hash;

/// One row the consumer lists: its key and what it draws.
#[derive(Debug, Clone, PartialEq)]
pub struct LeavingItem<K> {
    /// The row's identity, stable across renders (a notification id, a group's key).
    pub key: K,
    /// Its content.
    pub row: Element,
}

/// A list whose rows leave with an exit, in batches, and whose remaining rows heal by the
/// heights the leaving ones measured.
///
/// `items` is the whole list on every render; a key missing from it leaves. All the keys that
/// go missing in one render are one batch: a Clear drops a whole group's keys at once, then the
/// rows below heal by the group's summed height. `exit` is read on the render that starts a
/// batch, so a consumer may pass another exit for another kind of departure. The rows listed on
/// the first render are simply there. A key listed again while it leaves stays where it is.
/// `on_settled`
/// hears each dropped key once its batch has settled. `label` names the list
/// (`role=list`); each row is a `listitem`.
///
/// Keys are hashed to name the rows' nodes, so a key must hash the same on every render.
#[component]
pub fn LeavingList<K: Clone + PartialEq + Hash + 'static>(
    label: String,
    items: Vec<LeavingItem<K>>,
    #[props(default = Exit::Row)] exit: Exit,
    #[props(default)] on_settled: Option<EventHandler<K>>,
) -> Element {
    let rows = use_rows(&items);
    let keys: Vec<K> = items.iter().map(|item| item.key.clone()).collect();
    let roster = use_roster(
        keys,
        RosterSpec {
            leave: LeaveBy::Delist,
            exit,
            pitch: RowPitch::default(),
            on_settled: Some(EventHandler::new(move |key: K| {
                rows.forget(&key);
                if let Some(on_settled) = on_settled {
                    on_settled.call(key);
                }
            })),
        },
    );
    let pitches = roster.pitches();
    rsx! {
        div {
            class: "ds-leaving-list",
            role: "list",
            "aria-label": "{label}",
            for entry in roster.entries() {
                LeavingRow::<K> {
                    key: "{node_key(&entry.key)}",
                    entry: entry.clone(),
                    pitches,
                    row: rows.of(&entry.key),
                }
            }
        }
    }
}

/// A row's node key: its key's hash, in hex.
fn node_key<K: Hash>(key: &K) -> String {
    use std::hash::{BuildHasher, BuildHasherDefault, DefaultHasher};
    format!(
        "{:016x}",
        BuildHasherDefault::<DefaultHasher>::default().hash_one(key)
    )
}

/// The last content each key was listed with, so a row the consumer has dropped still draws
/// while it leaves.
struct Rows<K: 'static>(CopyValue<Vec<(K, Element)>>);

impl<K: 'static> Clone for Rows<K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K: 'static> Copy for Rows<K> {}

impl<K: Clone + PartialEq + 'static> Rows<K> {
    fn of(&self, key: &K) -> Element {
        self.0
            .peek()
            .iter()
            .find(|(held, _)| held == key)
            .map_or_else(|| rsx! {}, |(_, row)| row.clone())
    }

    fn forget(&self, key: &K) {
        let mut book = self.0;
        let _ = book
            .try_write()
            .map(|mut book| book.retain(|(held, _)| held != key));
    }
}

/// The book of rows, refreshed from this render's items (a listed row is always its newest
/// content; a dropped one keeps its last).
fn use_rows<K: Clone + PartialEq + 'static>(items: &[LeavingItem<K>]) -> Rows<K> {
    let mut book = use_hook(|| CopyValue::new(Vec::<(K, Element)>::new()));
    let kept: Vec<(K, Element)> = book
        .peek()
        .iter()
        .filter(|(key, _)| items.iter().all(|item| item.key != *key))
        .cloned()
        .collect();
    let next = items
        .iter()
        .map(|item| (item.key.clone(), item.row.clone()))
        .chain(kept)
        .collect();
    book.set(next);
    Rows(book)
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
