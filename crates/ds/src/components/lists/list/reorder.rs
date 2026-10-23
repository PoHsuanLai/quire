//! Reordering a `List` by dragging a row's grip: where each row stands while one is dragged, and
//! the pointer held to the rows, so a drag past the first or the last still lands at that end.
//! Pure, so the arithmetic is tested without a pointer; the list's own events drive it.

use ds_core::geometry::units::{Point, Px, Rect};
use ds_motion::drag::DragPhase;

/// Where one row stands while a row of the list is dragged.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) enum Dragged {
    /// Where it lies: nothing is dragged, or the drag does not pass it.
    #[default]
    Rest,
    /// The row being dragged, lifted and moved down by this much (up when negative) with the
    /// pointer.
    Lifted(Px),
    /// A row the dragged one has passed, moved by this much to make room for it.
    Moved(Px),
}

impl Dragged {
    /// `data-drag`, while the row is not at rest.
    pub(crate) fn slug(self) -> Option<&'static str> {
        match self {
            Dragged::Rest => None,
            Dragged::Lifted(_) => Some("lifted"),
            Dragged::Moved(_) => Some("moved"),
        }
    }

    /// The row's `--drag-y`, while it is not at rest.
    pub(crate) fn offset(self) -> Option<String> {
        match self {
            Dragged::Rest => None,
            Dragged::Lifted(dy) | Dragged::Moved(dy) => Some(format!("--drag-y:{}px", dy.0)),
        }
    }
}

/// Where the row at `index` of `order` stands in `phase`, the rows having measured `rects` (in
/// `order`'s order) when the drag began, which started at `from`. The dragged row follows the
/// pointer; the rows between where it was and where it would land move one of its heights
/// toward the place it left.
pub(crate) fn stand<K: PartialEq>(
    phase: &DragPhase<K>,
    order: &[K],
    rects: &[Rect],
    from: Point,
    index: usize,
) -> Dragged {
    let DragPhase::Live { key, at, target } = phase else {
        return Dragged::Rest;
    };
    let Some(source) = order.iter().position(|held| held == key) else {
        return Dragged::Rest;
    };
    if index == source {
        return Dragged::Lifted(at.y - from.y);
    }
    let (Some(to), Some(lifted)) = (*target, rects.get(source)) else {
        return Dragged::Rest;
    };
    let height = lifted.size.height;
    if source < to && (source + 1..=to).contains(&index) {
        Dragged::Moved(Px(-height.0))
    } else if to < source && (to..source).contains(&index) {
        Dragged::Moved(height)
    } else {
        Dragged::Rest
    }
}

/// `at` held inside the rows `rects` (in order, one under the other): a pointer above the first
/// is on the first, one below the last on the last, one beside them level with where it is.
pub(crate) fn held_to(at: Point, rects: &[Rect]) -> Point {
    let (Some(first), Some(last)) = (rects.first(), rects.last()) else {
        return at;
    };
    let inside = |value: Px, low: Px, high: Px| Px(value.0.clamp(low.0, (high.0 - 1.0).max(low.0)));
    Point {
        x: inside(at.x, first.left(), first.right()),
        y: inside(at.y, first.top(), last.bottom()),
    }
}

#[cfg(test)]
mod tests {
    use super::{Dragged, held_to, stand};
    use ds_core::geometry::units::{Point, Px, Rect, Size};
    use ds_motion::drag::DragPhase;

    fn point(x: f32, y: f32) -> Point {
        Point { x: Px(x), y: Px(y) }
    }

    /// Four rows 40 high, one under the other from y 100.
    fn rows() -> Vec<Rect> {
        (0..4)
            .map(|n| Rect {
                origin: point(0.0, 100.0 + 40.0 * n as f32),
                size: Size {
                    width: Px(300.0),
                    height: Px(40.0),
                },
            })
            .collect()
    }

    #[test]
    fn the_dragged_row_follows_the_pointer_and_the_rows_it_passes_make_room() {
        let order = ['a', 'b', 'c', 'd'];
        let from = point(10.0, 120.0);
        // 'a' dragged down onto 'c': b and c move up one row, d stays.
        let down = DragPhase::Live {
            key: 'a',
            at: point(10.0, 205.0),
            target: Some(2),
        };
        let want = [
            Dragged::Lifted(Px(85.0)),
            Dragged::Moved(Px(-40.0)),
            Dragged::Moved(Px(-40.0)),
            Dragged::Rest,
        ];
        for (index, want) in want.into_iter().enumerate() {
            assert_eq!(stand(&down, &order, &rows(), from, index), want, "{index}");
        }
        // 'd' dragged up onto 'b': b and c move down one row, a stays.
        let up = DragPhase::Live {
            key: 'd',
            at: point(10.0, 150.0),
            target: Some(1),
        };
        let from = point(10.0, 230.0);
        let want = [
            Dragged::Rest,
            Dragged::Moved(Px(40.0)),
            Dragged::Moved(Px(40.0)),
            Dragged::Lifted(Px(-80.0)),
        ];
        for (index, want) in want.into_iter().enumerate() {
            assert_eq!(stand(&up, &order, &rows(), from, index), want, "{index}");
        }
    }

    #[test]
    fn nothing_moves_until_a_drag_is_live() {
        let order = ['a', 'b'];
        let at = point(0.0, 0.0);
        for phase in [DragPhase::Idle, DragPhase::Pending { key: 'a', from: at }] {
            for index in 0..2 {
                assert_eq!(stand(&phase, &order, &rows(), at, index), Dragged::Rest);
            }
        }
    }

    #[test]
    fn a_pointer_past_the_rows_is_held_to_the_nearest() {
        let rows = rows();
        // (pointer, held to)
        let cases = [
            (point(10.0, 50.0), point(10.0, 100.0)),
            (point(10.0, 900.0), point(10.0, 259.0)),
            (point(-20.0, 150.0), point(0.0, 150.0)),
            (point(500.0, 150.0), point(299.0, 150.0)),
            (point(10.0, 150.0), point(10.0, 150.0)),
        ];
        for (at, want) in cases {
            assert_eq!(held_to(at, &rows), want, "{at:?}");
        }
        assert_eq!(held_to(point(3.0, 4.0), &[]), point(3.0, 4.0));
    }

    #[test]
    fn a_row_at_rest_writes_nothing() {
        assert_eq!(Dragged::Rest.slug(), None);
        assert_eq!(Dragged::Rest.offset(), None);
        assert_eq!(Dragged::Lifted(Px(12.5)).slug(), Some("lifted"));
        assert_eq!(
            Dragged::Moved(Px(-40.0)).offset().as_deref(),
            Some("--drag-y:-40px")
        );
    }
}
