//! What a key means to a list, pure (design/30 section 1.4, roving focus and type-to-select): the
//! arrows and Home and End move the cursor and stop at the ends, Enter and Space pick, letters
//! jump to the next label that starts with them.

use crate::components::lists::list::model::{ListItem, ListRole};
use crate::stack::roving::{Rove, Roving, Wrap};
use dioxus::prelude::{Key, Modifiers};
use ds_core::command::command_keys;
use ds_core::vocab::Availability;

/// What a key asks of a list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ListKey {
    /// The cursor moves.
    Move(Rove),
    /// The row under the cursor is picked.
    Pick,
    /// Letters typed: the cursor jumps to the next label that starts with them.
    Jump(String),
}

/// The list's reading of `key`; `None` for a key it leaves alone.
pub(crate) fn list_key(key: &Key, modifiers: Modifiers) -> Option<ListKey> {
    let chord =
        command_keys(modifiers).intersects(Modifiers::CONTROL | Modifiers::ALT | Modifiers::META);
    match key {
        Key::ArrowDown => Some(ListKey::Move(Rove::of(&Key::ArrowDown)?)),
        Key::ArrowUp => Some(ListKey::Move(Rove::of(&Key::ArrowUp)?)),
        Key::Home | Key::End => Some(ListKey::Move(Rove::of(key)?)),
        Key::Enter => Some(ListKey::Pick),
        Key::Character(text) if text == " " && !chord => Some(ListKey::Pick),
        Key::Character(text) if !chord && !text.trim().is_empty() => {
            Some(ListKey::Jump(text.clone()))
        }
        _ => None,
    }
}

/// The stops of `items`: each row's index in `items`, its key, and whether the keys may rest on it.
fn stops<K: Clone>(items: &[ListItem<K>]) -> Vec<(usize, K, Availability)> {
    items
        .iter()
        .enumerate()
        .filter(|(_, item)| item.role == ListRole::Row)
        .map(|(at, item)| (at, item.key.clone(), item.availability))
        .collect()
}

/// The key the cursor moves to from `cursor` by `rove`: the list stops at its ends and passes
/// over disabled rows. `None` when there is no row to rest on.
pub(crate) fn moved<K: Clone + PartialEq>(
    items: &[ListItem<K>],
    cursor: Option<&K>,
    rove: Rove,
) -> Option<K> {
    let set = Roving::new(
        stops(items)
            .into_iter()
            .map(|(_, key, availability)| (key, availability))
            .collect(),
        Wrap::Stops,
    );
    let set = match cursor {
        Some(key) => set.focus(key),
        None => set,
    };
    set.rove(rove).focused().cloned()
}

/// The key the cursor jumps to when `text` is typed with the cursor on `cursor`, given the
/// buffer's next state comes from `find`: it is handed the labels of the enabled rows and the
/// position of the cursor among them.
pub(crate) fn labelled<K: Clone + PartialEq>(
    items: &[ListItem<K>],
    cursor: Option<&K>,
    find: impl FnOnce(&[&str], usize) -> Option<usize>,
) -> Option<K> {
    let rows: Vec<&ListItem<K>> = items
        .iter()
        .filter(|item| item.role == ListRole::Row)
        .collect();
    let labels: Vec<&str> = rows
        .iter()
        .map(|item| match item.availability {
            Availability::Enabled => item.label.as_str(),
            Availability::Disabled | Availability::Busy => "",
        })
        .collect();
    let from = cursor
        .and_then(|key| rows.iter().position(|item| item.key == *key))
        .unwrap_or(labels.len().saturating_sub(1));
    find(&labels, from)
        .and_then(|at| rows.get(at))
        .map(|item| item.key.clone())
}

#[cfg(test)]
mod tests {
    use super::{ListKey, labelled, list_key, moved};
    use crate::components::lists::list::model::ListItem;
    use crate::stack::roving::{Edge, Rove, Step};
    use dioxus::prelude::{Element, Key, Modifiers, VNode};
    use ds_core::vocab::Availability;

    fn items() -> Vec<ListItem<u8>> {
        let blank = || -> Element { VNode::empty() };
        vec![
            ListItem::heading(0, blank()),
            ListItem::row(1, "Archive", blank()),
            ListItem::row(2, "Move", blank()).with(Availability::Disabled),
            ListItem::row(3, "Mute", blank()),
        ]
    }

    #[test]
    fn keys_read_as_moves_picks_and_jumps() {
        let none = Modifiers::empty();
        let cases: Vec<(Key, Modifiers, Option<ListKey>)> = vec![
            (
                Key::ArrowDown,
                none,
                Some(ListKey::Move(Rove::Step(Step::Down))),
            ),
            (
                Key::ArrowUp,
                none,
                Some(ListKey::Move(Rove::Step(Step::Up))),
            ),
            (
                Key::Home,
                none,
                Some(ListKey::Move(Rove::Edge(Edge::First))),
            ),
            (Key::End, none, Some(ListKey::Move(Rove::Edge(Edge::Last)))),
            (Key::Enter, none, Some(ListKey::Pick)),
            (Key::Character(" ".into()), none, Some(ListKey::Pick)),
            (
                Key::Character("m".into()),
                none,
                Some(ListKey::Jump("m".into())),
            ),
            (Key::Character("m".into()), Modifiers::CONTROL, None),
            (Key::ArrowLeft, none, None),
        ];
        for (key, modifiers, want) in cases {
            assert_eq!(list_key(&key, modifiers), want, "{key:?} {modifiers:?}");
        }
    }

    #[test]
    fn the_cursor_stops_at_the_ends_and_passes_headings_and_disabled_rows() {
        let items = items();
        let down = Rove::Step(Step::Down);
        let up = Rove::Step(Step::Up);
        let cases: &[(Option<u8>, Rove, Option<u8>)] = &[
            (None, down, Some(1)),
            (None, up, Some(3)),
            (Some(1), down, Some(3)),
            (Some(3), down, Some(3)),
            (Some(3), up, Some(1)),
            (Some(1), up, Some(1)),
            (Some(3), Rove::Edge(Edge::First), Some(1)),
            (Some(1), Rove::Edge(Edge::Last), Some(3)),
        ];
        for &(cursor, rove, want) in cases {
            assert_eq!(
                moved(&items, cursor.as_ref(), rove),
                want,
                "{cursor:?} {rove:?}"
            );
        }
    }

    #[test]
    fn typing_jumps_over_the_enabled_labels() {
        let items = items();
        let found = labelled(&items, Some(&1), |labels, from| {
            assert_eq!(labels, ["Archive", "", "Mute"]);
            assert_eq!(from, 0);
            Some(2)
        });
        assert_eq!(found, Some(3));
    }
}
