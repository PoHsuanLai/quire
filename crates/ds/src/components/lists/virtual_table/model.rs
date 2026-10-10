//! What a `VirtualTable` decides, pure: which row a key moves the cursor to.

use crate::components::lists::list::keys::{ListKey, list_key};
use crate::components::lists::virtual_list::model::roved;
use crate::stack::roving::{Rove, Step};
use dioxus::prelude::{Key, Modifiers};

/// What a key asks of a virtual table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TableKey {
    /// The cursor moves by a row, or to an end.
    Move(Rove),
    /// The cursor moves by a page.
    Page(Step),
    /// The row under the cursor is picked.
    Pick,
}

/// The table's reading of `key`; `None` for a key it leaves alone. Letters are not read: the rows
/// are not at hand to match a label against.
pub(crate) fn table_key(key: &Key, modifiers: Modifiers) -> Option<TableKey> {
    match key {
        Key::PageDown => Some(TableKey::Page(Step::Down)),
        Key::PageUp => Some(TableKey::Page(Step::Up)),
        _ => match list_key(key, modifiers)? {
            ListKey::Move(rove) => Some(TableKey::Move(rove)),
            ListKey::Pick => Some(TableKey::Pick),
            ListKey::Jump(_) => None,
        },
    }
}

/// Where the cursor goes from `at` among `len` rows for `key`, a page being `page` rows; `None`
/// for an empty table or a key that does not move it. The cursor stops at the ends. A page with
/// no cursor starts from the first row going down and the last going up.
pub(crate) fn target(at: Option<usize>, len: usize, key: TableKey, page: usize) -> Option<usize> {
    let last = len.checked_sub(1)?;
    let page = page.max(1);
    match key {
        TableKey::Move(rove) => roved(at, len, rove),
        TableKey::Page(Step::Down) => Some(at.map_or(0, |at| (at + page).min(last)).min(last)),
        TableKey::Page(Step::Up) => Some(at.map_or(last, |at| at.saturating_sub(page)).min(last)),
        TableKey::Pick => None,
    }
}

/// How many whole rows of `pitch` the viewport `height` shows, at least one.
pub(crate) fn page_rows(height: f32, pitch: f32) -> usize {
    match pitch > 0.0 && height.is_finite() {
        true => ((height / pitch).floor() as usize).max(1),
        false => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::{TableKey, page_rows, table_key, target};
    use crate::stack::roving::{Edge, Rove, Step};
    use dioxus::prelude::{Key, Modifiers};

    #[test]
    fn the_cursor_moves_by_a_row_an_end_or_a_page_and_stops_at_the_ends() {
        let down = TableKey::Move(Rove::Step(Step::Down));
        let up = TableKey::Move(Rove::Step(Step::Up));
        let end = TableKey::Move(Rove::Edge(Edge::Last));
        let page_down = TableKey::Page(Step::Down);
        let page_up = TableKey::Page(Step::Up);
        // (cursor, rows, key, page, wanted)
        let cases: &[(Option<usize>, usize, TableKey, usize, Option<usize>)] = &[
            (None, 100, down, 10, Some(0)),
            (Some(5), 100, down, 10, Some(6)),
            (Some(99), 100, down, 10, Some(99)),
            (Some(5), 100, up, 10, Some(4)),
            (Some(5), 100, end, 10, Some(99)),
            (Some(5), 100, page_down, 10, Some(15)),
            (Some(95), 100, page_down, 10, Some(99)),
            (Some(5), 100, page_up, 10, Some(0)),
            (Some(50), 100, page_up, 10, Some(40)),
            (None, 100, page_down, 10, Some(0)),
            (None, 100, page_up, 10, Some(99)),
            (Some(5), 100, page_down, 0, Some(6)),
            (Some(150), 100, page_down, 10, Some(99)),
            (Some(0), 0, page_down, 10, None),
            (Some(3), 100, TableKey::Pick, 10, None),
        ];
        for &(at, len, key, page, want) in cases {
            assert_eq!(target(at, len, key, page), want, "{at:?} of {len} {key:?}");
        }
    }

    #[test]
    fn a_key_is_read_as_a_move_a_page_or_a_pick_and_letters_are_left_alone() {
        let cases: &[(Key, Option<TableKey>)] = &[
            (Key::PageDown, Some(TableKey::Page(Step::Down))),
            (Key::PageUp, Some(TableKey::Page(Step::Up))),
            (Key::ArrowDown, Some(TableKey::Move(Rove::Step(Step::Down)))),
            (Key::Home, Some(TableKey::Move(Rove::Edge(Edge::First)))),
            (Key::Enter, Some(TableKey::Pick)),
            (Key::Character("a".to_owned()), None),
            (Key::Tab, None),
        ];
        for (key, want) in cases {
            assert_eq!(table_key(key, Modifiers::empty()), *want, "{key:?}");
        }
    }

    #[test]
    fn a_page_is_the_whole_rows_the_viewport_shows() {
        // (viewport height, pitch, wanted)
        let cases = [
            (168.0, 32.0, 5),
            (160.0, 32.0, 5),
            (10.0, 32.0, 1),
            (0.0, 32.0, 1),
            (100.0, 0.0, 1),
            (f32::NAN, 32.0, 1),
        ];
        for (height, pitch, want) in cases {
            assert_eq!(page_rows(height, pitch), want, "{height} over {pitch}");
        }
    }
}
