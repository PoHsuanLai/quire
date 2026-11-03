//! What a `VirtualList` decides, pure: its row heights and what a removal means, which keys left
//! the list and where their rows were, how far the rows below a dropped row heal, and where the
//! keys move the cursor.

use crate::components::lists::virtual_list::layout::Layout;
use crate::stack::roving::{Edge, Rove, Step};
use dioxus::prelude::Callback;
use ds_core::geometry::scroll::Scroll;
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

/// How far to scroll so the rows the viewer is looking at stay where they are when the keys
/// change from `old` (laid out as `before`) to `new` (laid out as `after`): the first row that
/// shows whole in `seen` and is still listed is the anchor, and the shift is how far its top moved.
/// `None` at the top of the list (new rows arriving there should show), when no row shows whole
/// that is still listed, or when the anchor did not move.
pub(crate) fn anchored<K: Eq + Hash>(
    old: &[K],
    before: &Layout,
    new: &[K],
    after: &Layout,
    seen: Scroll,
) -> Option<Px> {
    if seen.offset.0 <= 0.0 || old.is_empty() {
        return None;
    }
    let bottom = seen.offset.0 + seen.viewport.0;
    let index: HashMap<&K, usize> = new.iter().enumerate().map(|(at, key)| (key, at)).collect();
    let first = (0..old.len()).find(|at| before.top(*at).0 >= seen.offset.0)?;
    (first..old.len())
        .take_while(|at| before.top(*at + 1).0 <= bottom)
        .find_map(|at| index.get(&old[at]).map(|now| (at, *now)))
        .and_then(|(at, now)| {
            let shift = after.top(now).0 - before.top(at).0;
            (shift != 0.0).then_some(Px(shift))
        })
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
    use super::{Departed, anchored, departed, roved};
    use crate::components::lists::virtual_list::layout::Layout;
    use crate::stack::roving::{Edge, Rove, Step};
    use ds_core::geometry::scroll::Scroll;
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

    fn even(len: usize) -> Layout {
        Layout::Even {
            pitch: Px(20.0),
            len,
        }
    }

    fn seen(offset: f32) -> Scroll {
        Scroll {
            offset: Px(offset),
            viewport: Px(100.0),
            content: Px(1000.0),
        }
    }

    #[test]
    fn rows_arriving_above_shift_the_offset_by_how_far_the_anchor_moved() {
        let old: Vec<u32> = (10..40).collect();
        let mut new: Vec<u32> = vec![1, 2, 3];
        new.extend(&old);
        // Scrolled to 205: row index 11 (top 220) is the first whole row.
        let got = anchored(&old, &even(30), &new, &even(33), seen(205.0));
        assert_eq!(got, Some(Px(60.0)));
        // Exactly on a row: that row anchors.
        let got = anchored(&old, &even(30), &new, &even(33), seen(200.0));
        assert_eq!(got, Some(Px(60.0)));
    }

    #[test]
    fn nothing_shifts_at_the_top_or_when_nothing_moved() {
        let old: Vec<u32> = (10..40).collect();
        let mut new: Vec<u32> = vec![1, 2, 3];
        new.extend(&old);
        assert_eq!(anchored(&old, &even(30), &new, &even(33), seen(0.0)), None);
        assert_eq!(
            anchored(&old, &even(30), &old, &even(30), seen(205.0)),
            None
        );
        // Rows added below the viewer move nothing above them.
        let mut below = old.clone();
        below.push(99);
        assert_eq!(
            anchored(&old, &even(30), &below, &even(31), seen(205.0)),
            None
        );
    }

    #[test]
    fn a_removed_anchor_falls_to_the_next_whole_row_that_is_still_listed() {
        let old: Vec<u32> = (0..30).collect();
        // Rows 0 and 11 are gone (11 is the first whole row at 205), 12 anchors.
        let new: Vec<u32> = old
            .iter()
            .copied()
            .filter(|k| *k != 0 && *k != 11)
            .collect();
        let got = anchored(&old, &even(30), &new, &even(28), seen(205.0));
        assert_eq!(got, Some(Px(-40.0)));
    }

    #[test]
    fn a_row_leaving_below_a_still_anchor_moves_nothing() {
        let old: Vec<u32> = (0..30).collect();
        // 11 and 12 show whole at 205 and stay put; 13 leaves, so 14 rises, but 11 anchors.
        let gone: Vec<u32> = old.iter().copied().filter(|k| *k != 13).collect();
        assert_eq!(
            anchored(&old, &even(30), &gone, &even(29), seen(205.0)),
            None
        );
        // A row arriving among the visible rows, below the anchor, moves nothing either.
        let mut arrived = old.clone();
        arrived.insert(13, 99);
        assert_eq!(
            anchored(&old, &even(30), &arrived, &even(31), seen(205.0)),
            None
        );
    }

    #[test]
    fn nothing_shifts_when_no_whole_visible_row_is_still_listed() {
        let old: Vec<u32> = (0..30).collect();
        let new: Vec<u32> = (100..130).collect();
        assert_eq!(
            anchored(&old, &even(30), &new, &even(30), seen(205.0)),
            None
        );
    }
}
