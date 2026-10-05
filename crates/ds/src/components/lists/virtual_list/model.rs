//! What a `VirtualList` decides, pure: its row heights and what a removal means, which keys left
//! the list and where their rows were, how far the rows below a dropped row heal, and where the
//! keys move the cursor.

use crate::components::lists::virtual_list::layout::Layout;
use crate::stack::roving::{Edge, Rove, Step};
use dioxus::prelude::Callback;
use ds_core::geometry::units::Px;
use ds_motion::anim::Anim;
use ds_motion::presence::Exit;
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

/// The animation `exit` plays, whose length the list waits out before it drops the rows.
pub(crate) fn exit_anim(exit: Exit) -> Anim {
    match exit {
        Exit::Row => Anim::RowOut,
        Exit::OsdOut => Anim::OsdOut,
        Exit::PaneOut => Anim::PaneOutR,
        Exit::Fade => Anim::MenuOut,
        Exit::SheetOut => Anim::SheetOut,
        Exit::PanelOut => Anim::PanelOut,
    }
}

/// A row that left the list while it was mounted: it plays its exit where it stood.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Leaving<K> {
    /// The key it had.
    pub(crate) key: K,
    /// The index, in the new list, of the row it stood above: it is drawn before that row.
    pub(crate) slot: usize,
    /// How tall its row was.
    pub(crate) height: Px,
}

/// Where each key of `keys` is.
pub(crate) fn positions<K: Eq + Hash>(keys: &[K]) -> HashMap<&K, usize> {
    keys.iter().enumerate().map(|(at, key)| (key, at)).collect()
}

/// The slot in `new` (whose keys sit at `index`, `new_len` of them) of a row that stood above
/// `old[from]`: the index of the first key at or after `from` in `old` that `new` still has, or
/// `new_len` when none.
pub(crate) fn slot_in<K: Eq + Hash>(
    old: &[K],
    from: usize,
    index: &HashMap<&K, usize>,
    new_len: usize,
) -> usize {
    old.get(from..)
        .unwrap_or_default()
        .iter()
        .find_map(|later| index.get(later).copied())
        .unwrap_or(new_len)
}

/// The mounted rows `old[shown]` that `new` no longer lists, each with its slot in `new`: the
/// index of the first key after it in `old` that `new` still has, or `new.len()` when none. A
/// row is as tall as `before` (the layout of `old`) says.
pub(crate) fn leaving<K: Clone + Eq + Hash>(
    old: &[K],
    shown: Range<usize>,
    new: &[K],
    before: &Layout,
) -> Vec<Leaving<K>> {
    let index = positions(new);
    old.iter()
        .enumerate()
        .take(shown.end)
        .skip(shown.start)
        .filter(|(_, key)| !index.contains_key(key))
        .map(|(at, key)| Leaving {
            key: key.clone(),
            slot: slot_in(old, at + 1, &index, new.len()),
            height: before.height(at),
        })
        .collect()
}

/// How far the row at `index` starts below its resting place once the rows `dropped` (each a
/// slot and the height it had) are gone: the height of each dropped row above it. `None` for a
/// row nothing was dropped above.
pub(crate) fn heal_dy(dropped: &[(usize, Px)], index: usize) -> Option<Px> {
    let above = dropped.iter().filter(|(slot, _)| *slot <= index);
    let count = above.clone().count();
    (count > 0).then(|| Px(above.map(|(_, height)| height.0).sum()))
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
    use super::{Leaving, heal_dy, leaving, positions, roved, slot_in};
    use crate::components::lists::virtual_list::layout::Layout;
    use crate::stack::roving::{Edge, Rove, Step};
    use ds_core::geometry::units::Px;

    /// A case: name, old keys, mounted old indexes, new keys, wanted leavers.
    type Case = (
        &'static str,
        Vec<u32>,
        std::ops::Range<usize>,
        Vec<u32>,
        Vec<Leaving<u32>>,
    );

    fn gone(key: u32, slot: usize) -> Leaving<u32> {
        Leaving {
            key,
            slot,
            height: Px(20.0),
        }
    }

    #[test]
    fn a_removed_mounted_row_leaves_above_the_row_that_followed_it() {
        // (name, old keys, mounted old indexes, new keys, wanted)
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
                vec![gone(2, 1)],
            ),
            (
                "two adjacent",
                vec![1, 2, 3, 4],
                0..4,
                vec![1, 4],
                vec![gone(2, 1), gone(3, 1)],
            ),
            (
                "two apart",
                vec![1, 2, 3, 4],
                0..4,
                vec![2, 4],
                vec![gone(1, 0), gone(3, 1)],
            ),
            (
                "the last",
                vec![1, 2, 3, 4],
                0..4,
                vec![1, 2, 3],
                vec![gone(4, 3)],
            ),
            (
                "all",
                vec![1, 2],
                0..2,
                vec![],
                vec![gone(1, 0), gone(2, 0)],
            ),
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
                vec![gone(2, 1)],
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
            assert_eq!(leaving(&old, shown, &new, &before), want, "{name}");
        }
    }

    #[test]
    fn a_leaving_row_keeps_standing_above_the_row_it_stood_above_when_the_keys_change() {
        /// A case: name, keys it left from, its slot there, keys now, wanted slot.
        type SlotCase = (&'static str, &'static [u32], usize, &'static [u32], usize);
        let cases: &[SlotCase] = &[
            ("nothing changed", &[1, 2, 5], 2, &[1, 2, 5], 2),
            ("a key returns above it", &[1, 2, 5], 2, &[1, 2, 3, 5], 3),
            ("a key arrives below it", &[1, 2, 5], 2, &[1, 2, 5, 9], 2),
            (
                "the row it stood above is gone",
                &[1, 2, 5, 6],
                2,
                &[1, 2, 6],
                2,
            ),
            ("everything below it is gone", &[1, 2, 5], 2, &[1, 2], 2),
            ("it stood at the end", &[1, 2], 2, &[1, 2, 7], 3),
            (
                "a key arrives at the front",
                &[1, 2, 5],
                2,
                &[0, 1, 2, 5],
                3,
            ),
        ];
        for &(name, old, from, new, want) in cases {
            let index = positions(new);
            assert_eq!(slot_in(old, from, &index, new.len()), want, "{name}");
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
        let got = leaving(&[1, 2, 3], 0..3, &[1, 3], &before);
        assert_eq!(
            got,
            vec![Leaving {
                key: 2,
                slot: 1,
                height: Px(24.0)
            }]
        );
    }

    #[test]
    fn rows_below_dropped_rows_start_the_dropped_heights_lower() {
        let dropped = |rows: &[(usize, f32)]| -> Vec<(usize, Px)> {
            rows.iter()
                .map(|&(slot, height)| (slot, Px(height)))
                .collect()
        };
        // (dropped slots and heights, the row's index, wanted dy)
        /// A case: dropped slots and heights, the row's index, the wanted distance.
        type HealCase = (&'static [(usize, f32)], usize, Option<f32>);
        let cases: &[HealCase] = &[
            (&[], 5, None),
            (&[(3, 20.0)], 2, None),
            (&[(3, 20.0)], 3, Some(20.0)),
            (&[(3, 20.0)], 9, Some(20.0)),
            (&[(3, 20.0), (3, 20.0)], 3, Some(40.0)),
            (&[(3, 20.0), (6, 20.0)], 5, Some(20.0)),
            (&[(3, 20.0), (6, 20.0)], 6, Some(40.0)),
            (&[(3, 66.0), (6, 24.0)], 5, Some(66.0)),
            (&[(3, 66.0), (6, 24.0)], 6, Some(90.0)),
        ];
        for &(rows, index, want) in cases {
            assert_eq!(
                heal_dy(&dropped(rows), index),
                want.map(Px),
                "{rows:?} at {index}"
            );
        }
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
