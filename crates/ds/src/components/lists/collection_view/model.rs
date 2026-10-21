//! What a `CollectionView` decides, pure: what a key asks of the grid, which tile it moves the
//! cursor to, and which tiles the rows in the window hold.

use crate::components::lists::emoji_grid::nav::{GridMove, GridStep, grid_step};
use crate::components::lists::list::keys::{ListKey, list_key};
use crate::components::lists::virtual_list::model::roved;
use crate::stack::roving::{Edge, Rove};
use chordkit::Platform;
use dioxus::prelude::{Key, Modifiers};
use ds_core::command::holds_primary;
use std::ops::Range;

/// What a key asks of a collection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CollectionKey {
    /// The cursor moves by a tile along a row, or by a row down or up.
    Arrow(GridStep),
    /// The cursor moves to the first or the last tile.
    Edge(Edge),
    /// The primary modifier with Down (Command-Down, Finder's Open): the tile under the cursor is
    /// opened.
    Open,
    /// Letters typed: the cursor jumps to the next caption that starts with them.
    Jump(String),
}

/// The collection's reading of `key` on `platform`; `None` for a key it leaves alone. Left and
/// Right are the grid's own, so the list's reading (which has no sideways) is asked only for the
/// rest. As in `VirtualTable`, Shift alone still moves (and extends), any other modifier makes the
/// key the app's, and the one exception is the primary modifier with Down. Return and Space are
/// the app's too: in Finder they rename and Quick Look.
pub(crate) fn collection_key(
    key: &Key,
    modifiers: Modifiers,
    platform: Platform,
) -> Option<CollectionKey> {
    let chorded = modifiers
        .intersects(Modifiers::CONTROL | Modifiers::ALT | Modifiers::SUPER | Modifiers::META);
    if chorded {
        let primary_alone = holds_primary(platform, modifiers)
            && !modifiers.intersects(Modifiers::SHIFT | Modifiers::ALT);
        return (primary_alone && *key == Key::ArrowDown).then_some(CollectionKey::Open);
    }
    match key {
        Key::ArrowLeft => Some(CollectionKey::Arrow(GridStep::Left)),
        Key::ArrowRight => Some(CollectionKey::Arrow(GridStep::Right)),
        Key::ArrowUp => Some(CollectionKey::Arrow(GridStep::Up)),
        Key::ArrowDown => Some(CollectionKey::Arrow(GridStep::Down)),
        _ => match list_key(key, modifiers)? {
            ListKey::Move(Rove::Edge(edge)) => Some(CollectionKey::Edge(edge)),
            ListKey::Move(Rove::Step(_)) => None,
            ListKey::Pick => None,
            ListKey::Jump(text) => Some(CollectionKey::Jump(text)),
        },
    }
}

/// Where the cursor goes from `at` among `len` tiles in rows of `columns` for `key`; `None` for
/// an empty collection or a key that does not move it. The cursor stops at the edges. From no
/// cursor, Right and Down rest on the first tile and Left and Up on the last.
pub(crate) fn target(
    at: Option<usize>,
    len: usize,
    key: &CollectionKey,
    columns: usize,
) -> Option<usize> {
    let last = len.checked_sub(1)?;
    match (key, at) {
        (CollectionKey::Edge(edge), _) => roved(at, len, Rove::Edge(*edge)),
        (CollectionKey::Arrow(GridStep::Right | GridStep::Down), None) => Some(0),
        (CollectionKey::Arrow(GridStep::Left | GridStep::Up), None) => Some(last),
        (CollectionKey::Arrow(step), Some(at)) => {
            let across = u8::try_from(columns).unwrap_or(u8::MAX);
            Some(match grid_step(at, *step, len, across) {
                GridMove::To(next) => next,
                GridMove::Out(_) => at.min(last),
            })
        }
        (CollectionKey::Open | CollectionKey::Jump(_), _) => None,
    }
}

/// The tiles held by the rows `rows` of a grid `columns` wide among `len` tiles.
pub(crate) fn tiles_in(rows: Range<usize>, columns: usize, len: usize) -> Range<usize> {
    let end = rows.end.saturating_mul(columns).min(len);
    let start = rows.start.saturating_mul(columns).min(end);
    start..end
}

#[cfg(test)]
mod tests {
    use super::{CollectionKey, collection_key, target, tiles_in};
    use crate::components::lists::emoji_grid::nav::GridStep;
    use crate::stack::roving::Edge;
    use chordkit::Platform;
    use dioxus::prelude::{Key, Modifiers};

    #[test]
    fn a_key_is_read_as_an_arrow_an_edge_an_open_or_letters_and_the_rest_are_left_alone() {
        let cases: &[(Key, Option<CollectionKey>)] = &[
            (Key::ArrowLeft, Some(CollectionKey::Arrow(GridStep::Left))),
            (Key::ArrowRight, Some(CollectionKey::Arrow(GridStep::Right))),
            (Key::ArrowUp, Some(CollectionKey::Arrow(GridStep::Up))),
            (Key::ArrowDown, Some(CollectionKey::Arrow(GridStep::Down))),
            (Key::Home, Some(CollectionKey::Edge(Edge::First))),
            (Key::End, Some(CollectionKey::Edge(Edge::Last))),
            (Key::Enter, None),
            (
                Key::Character("r".to_owned()),
                Some(CollectionKey::Jump("r".to_owned())),
            ),
            (Key::Tab, None),
            (Key::PageDown, None),
        ];
        for (key, want) in cases {
            assert_eq!(
                collection_key(key, Modifiers::empty(), Platform::MacOs),
                want.clone(),
                "{key:?}"
            );
        }
    }

    #[test]
    fn the_cursor_moves_by_a_tile_and_by_a_row_wraps_across_rows_and_stops_at_the_edges() {
        use CollectionKey::{Arrow, Edge as To};
        use GridStep::{Down, Left, Right, Up};
        // 13 tiles, 5 wide: rows are 0..=4, 5..=9 and 10..=12.
        // (cursor, key, wanted)
        let cases: &[(Option<usize>, CollectionKey, Option<usize>)] = &[
            (None, Arrow(Right), Some(0)),
            (None, Arrow(Down), Some(0)),
            (None, Arrow(Left), Some(12)),
            (None, Arrow(Up), Some(12)),
            (Some(0), Arrow(Right), Some(1)),
            (Some(4), Arrow(Right), Some(5)),
            (Some(5), Arrow(Left), Some(4)),
            (Some(0), Arrow(Left), Some(0)),
            (Some(12), Arrow(Right), Some(12)),
            (Some(2), Arrow(Down), Some(7)),
            (Some(7), Arrow(Down), Some(12)),
            (Some(8), Arrow(Down), Some(12)),
            (Some(12), Arrow(Down), Some(12)),
            (Some(7), Arrow(Up), Some(2)),
            (Some(2), Arrow(Up), Some(2)),
            (Some(6), To(Edge::First), Some(0)),
            (Some(6), To(Edge::Last), Some(12)),
            (Some(6), CollectionKey::Open, None),
            (Some(6), CollectionKey::Jump("a".to_owned()), None),
        ];
        for (at, key, want) in cases {
            assert_eq!(target(*at, 13, key, 5), *want, "{at:?} {key:?}");
        }
        assert_eq!(target(Some(0), 0, &Arrow(Down), 5), None, "no tiles");
    }

    #[test]
    fn the_tiles_of_a_window_of_rows_stop_at_the_last_tile() {
        // (rows, columns, tiles, wanted)
        let cases = [
            (0..2, 5, 13, 0..10),
            (1..3, 5, 13, 5..13),
            (2..9, 5, 13, 10..13),
            (4..9, 5, 13, 13..13),
            (0..0, 5, 13, 0..0),
            (0..3, 4, 0, 0..0),
        ];
        for (rows, columns, len, want) in cases {
            assert_eq!(tiles_in(rows.clone(), columns, len), want, "{rows:?}");
        }
    }
}
