//! What a `VirtualList` decides, pure: its row heights and what a removal means, which keys left
//! the list and where their rows were, how far the rows below a dropped row heal, and where the
//! keys move the cursor.

use crate::components::lists::virtual_list::layout::Layout;
use crate::stack::roving::{Edge, Rove, Step};
use dioxus::prelude::Callback;
use ds_core::geometry::units::Px;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Range;

/// How tall the rows are.
#[derive(Debug, Clone, PartialEq)]
pub enum RowHeight<K> {
    /// Every row is this tall, gap included: the row at index `i` starts at `i * height`.
    Fixed(Px),
    /// Each key's row is as tall as the callback says, known up front (a mail row and a
    /// shorter section heading in one list). A height that is negative or not a finite number
    /// takes no room. The callback is asked for every key whenever the list renders, so it
    /// should be a lookup, not a measurement.
    PerKey(Callback<K, Px>),
}

impl<K: Clone + 'static> RowHeight<K> {
    /// Where the rows of `keys` lie; `previous` is kept when nothing moved.
    pub(crate) fn layout(&self, keys: &[K], previous: &Layout) -> Layout {
        match self {
            RowHeight::Fixed(pitch) => Layout::Even {
                pitch: *pitch,
                len: keys.len(),
            },
            RowHeight::PerKey(height) => {
                Layout::uneven(keys.iter().map(|key| height.call(key.clone())), previous)
            }
        }
    }
}

/// What a change of the list's keys is to the rows it removes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Change {
    /// An edit: a removed row that was mounted plays the list's exit where it stood and the rows
    /// below heal.
    #[default]
    Edit,
    /// A replacement (a new search answering a new question): removed rows are gone at once and
    /// the new rows simply take their place, with no exit.
    Replace,
}

/// A row that left the list while it was mounted: the key it had and how tall its row was.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Departed<K> {
    /// The key it had.
    pub(crate) key: K,
    /// How tall its row was.
    pub(crate) height: Px,
}

/// The mounted rows `old[shown]` that `new` no longer lists, each as tall as `before` (the layout
/// of `old`) says.
pub(crate) fn departed<K: Clone + Eq + Hash>(
    old: &[K],
    shown: Range<usize>,
    new: &[K],
    before: &Layout,
) -> Vec<Departed<K>> {
    let index: HashMap<&K, usize> = new.iter().enumerate().map(|(at, key)| (key, at)).collect();
    old.iter()
        .enumerate()
        .take(shown.end)
        .skip(shown.start)
        .filter(|(_, key)| !index.contains_key(key))
        .map(|(at, key)| Departed {
            key: key.clone(),
            height: before.height(at),
        })
        .collect()
}

/// Where the cursor goes from `at` for `rove` among `len` rows; `None` for an empty list. The
/// cursor stops at the ends.
pub(crate) fn roved(at: Option<usize>, len: usize, rove: Rove) -> Option<usize> {
    let last = len.checked_sub(1)?;
    Some(match (rove, at) {
        (Rove::Edge(Edge::First), _) => 0,
        (Rove::Edge(Edge::Last), _) => last,
        (Rove::Step(Step::Down), None) => 0,
        (Rove::Step(Step::Up), None) => last,
        (Rove::Step(Step::Down), Some(at)) => (at + 1).min(last),
        (Rove::Step(Step::Up), Some(at)) => at.saturating_sub(1).min(last),
    })
}

#[cfg(test)]
mod tests {
    use super::{Departed, departed, roved};
    use crate::components::lists::virtual_list::layout::Layout;
    use crate::stack::roving::{Edge, Rove, Step};
    use ds_core::geometry::units::Px;

    /// A case: name, old keys, mounted old indexes, new keys, wanted leavers.
    type Case = (
        &'static str,
        Vec<u32>,
        std::ops::Range<usize>,
        Vec<u32>,
        Vec<u32>,
    );

    #[test]
    fn a_removed_mounted_row_is_a_leaver_and_a_removed_unmounted_row_is_not() {
        let cases: Vec<Case> = vec![
            (
                "nothing removed",
                vec![1, 2, 3, 4],
                0..4,
                vec![1, 2, 3, 4],
                vec![],
            ),
            (
                "one in the middle",
                vec![1, 2, 3, 4],
                0..4,
                vec![1, 3, 4],
                vec![2],
            ),
            (
                "two adjacent",
                vec![1, 2, 3, 4],
                0..4,
                vec![1, 4],
                vec![2, 3],
            ),
            ("two apart", vec![1, 2, 3, 4], 0..4, vec![2, 4], vec![1, 3]),
            ("the last", vec![1, 2, 3, 4], 0..4, vec![1, 2, 3], vec![4]),
            ("all", vec![1, 2], 0..2, vec![], vec![1, 2]),
            (
                "not mounted: above the window",
                vec![1, 2, 3, 4, 5],
                2..4,
                vec![2, 3, 4, 5],
                vec![],
            ),
            (
                "not mounted: below the window",
                vec![1, 2, 3, 4, 5],
                0..2,
                vec![1, 2, 3, 4],
                vec![],
            ),
            (
                "mounted and removed, one not",
                vec![1, 2, 3, 4, 5],
                1..4,
                vec![1, 3, 4, 5],
                vec![2],
            ),
            (
                "a new key is no leaver",
                vec![1, 2],
                0..2,
                vec![1, 9, 2],
                vec![],
            ),
        ];
        for (name, old, shown, new, want) in cases {
            let before = Layout::Even {
                pitch: Px(20.0),
                len: old.len(),
            };
            let got: Vec<u32> = departed(&old, shown, &new, &before)
                .into_iter()
                .map(|gone| gone.key)
                .collect();
            assert_eq!(got, want, "{name}");
        }
    }

    #[test]
    fn a_leaving_row_remembers_how_tall_it_was() {
        let before = Layout::uneven(
            [Px(66.0), Px(24.0), Px(66.0)].into_iter(),
            &Layout::Even {
                pitch: Px(0.0),
                len: 0,
            },
        );
        let got = departed(&[1, 2, 3], 0..3, &[1, 3], &before);
        assert_eq!(
            got,
            vec![Departed {
                key: 2,
                height: Px(24.0)
            }]
        );
    }

    #[test]
    fn the_cursor_moves_by_one_and_stops_at_the_ends() {
        let down = Rove::Step(Step::Down);
        let up = Rove::Step(Step::Up);
        let (first, last) = (Rove::Edge(Edge::First), Rove::Edge(Edge::Last));
        // (cursor, rows, move, wanted)
        let cases: &[(Option<usize>, usize, Rove, Option<usize>)] = &[
            (None, 5, down, Some(0)),
            (None, 5, up, Some(4)),
            (Some(2), 5, down, Some(3)),
            (Some(2), 5, up, Some(1)),
            (Some(4), 5, down, Some(4)),
            (Some(0), 5, up, Some(0)),
            (Some(2), 5, first, Some(0)),
            (Some(2), 5, last, Some(4)),
            (Some(9), 5, down, Some(4)),
            (None, 0, down, None),
            (Some(0), 0, last, None),
        ];
        for &(at, len, rove, want) in cases {
            assert_eq!(roved(at, len, rove), want, "{at:?} of {len} {rove:?}");
        }
    }
}
