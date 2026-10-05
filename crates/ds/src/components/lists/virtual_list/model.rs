//! What a `VirtualList` decides, pure: its row heights, which keys left the list and where their
//! rows were, how far the rows below a dropped row heal, and where the keys move the cursor.

use crate::stack::roving::{Edge, Rove, Step};
use ds_core::geometry::units::Px;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Range;

/// How tall a row is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RowHeight {
    /// Every row is this tall, gap included: the row at index `i` starts at `i * height`.
    Fixed(Px),
}

impl RowHeight {
    /// The distance from one row's top to the next one's.
    pub fn pitch(self) -> Px {
        match self {
            RowHeight::Fixed(height) => height,
        }
    }
}

/// A row that left the list while it was mounted: it plays its exit where it stood.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Leaving<K> {
    /// The key it had.
    pub(crate) key: K,
    /// The index, in the new list, of the row it stood above: it is drawn before that row.
    pub(crate) slot: usize,
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
/// index of the first key after it in `old` that `new` still has, or `new.len()` when none.
pub(crate) fn leaving<K: Clone + Eq + Hash>(
    old: &[K],
    shown: Range<usize>,
    new: &[K],
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
        })
        .collect()
}

/// How far the row at `index` starts below its resting place once the rows at `slots` are
/// dropped: one pitch for each dropped row above it. `None` for a row nothing was dropped above.
pub(crate) fn heal_dy(slots: &[usize], index: usize, pitch: Px) -> Option<Px> {
    let above = slots.iter().filter(|slot| **slot <= index).count();
    (above > 0).then_some(Px(pitch.0 * above as f32))
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
        Leaving { key, slot }
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
            assert_eq!(leaving(&old, shown, &new), want, "{name}");
        }
    }

    #[test]
    fn a_leaving_row_keeps_standing_above_the_row_it_stood_above_when_the_keys_change() {
        // (name, keys it left from, its slot there, keys now, wanted slot)
        let cases: &[(&str, &[u32], usize, &[u32], usize)] = &[
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
    fn rows_below_a_dropped_row_start_one_pitch_lower_for_each_row_above() {
        let pitch = Px(20.0);
        // (slots of the dropped rows, the row's index, wanted dy)
        let cases: &[(&[usize], usize, Option<f32>)] = &[
            (&[], 5, None),
            (&[3], 2, None),
            (&[3], 3, Some(20.0)),
            (&[3], 9, Some(20.0)),
            (&[3, 3], 3, Some(40.0)),
            (&[3, 6], 5, Some(20.0)),
            (&[3, 6], 6, Some(40.0)),
        ];
        for &(slots, index, want) in cases {
            assert_eq!(
                heal_dy(slots, index, pitch),
                want.map(Px),
                "{slots:?} at {index}"
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
