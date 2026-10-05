//! Where a `VirtualList`'s rows lie, pure: rows of one height are arithmetic, rows of their own
//! heights are a prefix sum searched by bisection. Either way the list asks the same questions:
//! which rows the viewport shows, where a row starts and how tall it is, how long the whole
//! list is and how far from its end counts as near.

use ds_core::geometry::scroll::{Scroll, ScrollSpan};
use ds_core::geometry::units::Px;
use std::ops::Range;
use std::rc::Rc;

/// The tops of `len` rows, then the list's length: `len + 1` entries, each the sum of the heights
/// before it.
#[derive(Debug, Clone)]
pub(crate) struct Tops(Rc<[f32]>);

impl PartialEq for Tops {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0) || self.0 == other.0
    }
}

/// How far down each row starts.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Layout {
    /// `len` rows `pitch` apart: nothing is stored, a row's top is `index * pitch`.
    Even { pitch: Px, len: usize },
    /// Rows of their own heights.
    Uneven(Tops),
}

/// A height as the list draws it: never negative, and never not-a-number or infinite (a row with
/// such a height takes no room).
fn drawn(height: Px) -> f32 {
    match height.0.is_finite() {
        true => height.0.max(0.0),
        false => 0.0,
    }
}

impl Layout {
    /// Rows of the heights `heights`, in order. `previous` is returned as it is when it already
    /// holds exactly these, so the same layout is the same value across renders.
    pub(crate) fn uneven(heights: impl Iterator<Item = Px>, previous: &Layout) -> Layout {
        let tops: Vec<f32> = std::iter::once(0.0)
            .chain(heights.scan(0.0f32, |top, height| {
                *top += drawn(height);
                Some(*top)
            }))
            .collect();
        match previous {
            Layout::Uneven(held) if *held.0 == *tops => previous.clone(),
            Layout::Even { .. } | Layout::Uneven(_) => Layout::Uneven(Tops(tops.into())),
        }
    }

    /// How many rows there are.
    pub(crate) fn len(&self) -> usize {
        match self {
            Layout::Even { len, .. } => *len,
            Layout::Uneven(Tops(tops)) => tops.len() - 1,
        }
    }

    /// Where row `index` starts; the list's length for `len()`, and past it clamped there.
    pub(crate) fn top(&self, index: usize) -> Px {
        match self {
            Layout::Even { pitch, len } => Px(index.min(*len) as f32 * pitch.0),
            Layout::Uneven(Tops(tops)) => Px(tops[index.min(tops.len() - 1)]),
        }
    }

    /// How tall row `index` is; zero past the end.
    pub(crate) fn height(&self, index: usize) -> Px {
        match self {
            Layout::Even { pitch, len } => match index < *len {
                true => *pitch,
                false => Px(0.0),
            },
            Layout::Uneven(_) => Px(self.top(index + 1).0 - self.top(index).0),
        }
    }

    /// How long the whole list is.
    pub(crate) fn total(&self) -> Px {
        self.top(self.len())
    }

    /// How much of the list lies below rows `..end`.
    pub(crate) fn below(&self, end: usize) -> Px {
        match self {
            Layout::Even { pitch, len } => Px(len.saturating_sub(end) as f32 * pitch.0),
            Layout::Uneven(_) => Px(self.total().0 - self.top(end).0),
        }
    }

    /// Row `index`'s extent in the scroller's content.
    pub(crate) fn span(&self, index: usize) -> ScrollSpan {
        ScrollSpan {
            start: self.top(index).0,
            length: self.height(index).0,
        }
    }

    /// The rows that show in `seen`, plus `overscan` more on each side, as a range of indexes.
    /// Empty for no rows; a viewport not measured yet shows only the overscan below the offset.
    pub(crate) fn window(&self, seen: Scroll, overscan: usize) -> Range<usize> {
        match self {
            Layout::Even { pitch, len } => seen.rows(*pitch, *len, overscan),
            Layout::Uneven(Tops(tops)) => {
                let len = tops.len() - 1;
                if len == 0 {
                    return 0..0;
                }
                let rows = &tops[..len];
                let offset = seen.offset.0.max(0.0);
                let bottom = offset + seen.viewport.0.max(0.0);
                let first = rows.partition_point(|top| *top <= offset).saturating_sub(1);
                let past = rows.partition_point(|top| *top < bottom);
                first.saturating_sub(overscan)..past.max(first).saturating_add(overscan).min(len)
            }
        }
    }

    /// The distance from the list's end that is near: the height of its last `rows` rows.
    pub(crate) fn page(&self, rows: usize) -> Px {
        match self {
            Layout::Even { pitch, .. } => Px(rows as f32 * pitch.0),
            Layout::Uneven(_) => Px(self.total().0 - self.top(self.len().saturating_sub(rows)).0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Layout;
    use ds_core::geometry::scroll::Scroll;
    use ds_core::geometry::units::Px;

    fn uneven(heights: &[f32]) -> Layout {
        let none = Layout::Even {
            pitch: Px(0.0),
            len: 0,
        };
        Layout::uneven(heights.iter().copied().map(Px), &none)
    }

    fn seen(offset: f32, viewport: f32) -> Scroll {
        Scroll {
            offset: Px(offset),
            viewport: Px(viewport),
            content: Px(0.0),
        }
    }

    #[test]
    fn the_tops_are_the_prefix_sums_of_the_heights() {
        // (name, heights, wanted tops of every row and then the total)
        let cases: &[(&str, &[f32], &[f32])] = &[
            ("no rows", &[], &[0.0]),
            ("one row", &[30.0], &[0.0, 30.0]),
            ("mixed", &[66.0, 24.0, 66.0], &[0.0, 66.0, 90.0, 156.0]),
            (
                "a zero row takes no room",
                &[10.0, 0.0, 10.0],
                &[0.0, 10.0, 10.0, 20.0],
            ),
            (
                "a negative row takes no room",
                &[10.0, -5.0, 10.0],
                &[0.0, 10.0, 10.0, 20.0],
            ),
            (
                "not-a-number takes no room",
                &[10.0, f32::NAN, 10.0],
                &[0.0, 10.0, 10.0, 20.0],
            ),
            (
                "infinity takes no room",
                &[10.0, f32::INFINITY, 10.0],
                &[0.0, 10.0, 10.0, 20.0],
            ),
        ];
        for &(name, heights, want) in cases {
            let layout = uneven(heights);
            let got: Vec<f32> = (0..=heights.len()).map(|at| layout.top(at).0).collect();
            assert_eq!(got, want, "{name}");
            assert_eq!(layout.len(), heights.len(), "{name}: len");
            assert_eq!(
                layout.total(),
                Px(*want.last().unwrap_or(&0.0)),
                "{name}: total"
            );
        }
    }

    #[test]
    fn what_lies_below_a_window_is_the_rest_of_the_list() {
        let layout = uneven(&[66.0, 24.0, 66.0]);
        // (end of the window, wanted)
        for (end, want) in [(0, 156.0), (1, 90.0), (2, 66.0), (3, 0.0), (9, 0.0)] {
            assert_eq!(layout.below(end), Px(want), "below {end}");
        }
        let even = Layout::Even {
            pitch: Px(20.0),
            len: 5,
        };
        assert_eq!(
            (even.below(2), even.below(5), even.below(9)),
            (Px(60.0), Px(0.0), Px(0.0))
        );
    }

    #[test]
    fn a_row_knows_its_own_height_and_span() {
        let layout = uneven(&[66.0, 24.0, 66.0]);
        // (index, wanted height, wanted start)
        for (index, height, start) in [
            (0, 66.0, 0.0),
            (1, 24.0, 66.0),
            (2, 66.0, 90.0),
            (3, 0.0, 156.0),
        ] {
            assert_eq!(layout.height(index), Px(height), "height {index}");
            let span = layout.span(index);
            assert_eq!((span.start, span.length), (start, height), "span {index}");
        }
    }

    #[test]
    fn the_window_is_the_rows_the_viewport_touches_plus_the_overscan() {
        // Rows of 10, 30, 10, 30, 10, 30, 10, 30: tops 0 10 40 50 80 90 120 130, total 160.
        let layout = uneven(&[10.0, 30.0, 10.0, 30.0, 10.0, 30.0, 10.0, 30.0]);
        // (name, offset, viewport, overscan, wanted rows)
        let cases: &[(&str, f32, f32, usize, std::ops::Range<usize>)] = &[
            ("the first row", 0.0, 10.0, 0, 0..1),
            ("the first two", 0.0, 11.0, 0, 0..2),
            ("a boundary starts the next row", 10.0, 10.0, 0, 1..2),
            ("inside a tall row", 20.0, 5.0, 0, 1..2),
            ("across three rows", 5.0, 40.0, 0, 0..3),
            ("the last row", 130.0, 30.0, 0, 7..8),
            ("to the very end", 100.0, 60.0, 0, 5..8),
            ("overscan clamps at the top", 0.0, 10.0, 3, 0..4),
            ("overscan clamps at the bottom", 130.0, 30.0, 3, 4..8),
            ("overscan on both sides", 50.0, 20.0, 1, 2..5),
            ("not measured: only the overscan", 0.0, 0.0, 2, 0..2),
            ("negative offset acts as the top", -50.0, 10.0, 0, 0..1),
        ];
        for (name, offset, viewport, overscan, want) in cases {
            assert_eq!(
                &layout.window(seen(*offset, *viewport), *overscan),
                want,
                "{name}"
            );
        }
        assert_eq!(uneven(&[]).window(seen(0.0, 100.0), 4), 0..0, "no rows");
    }

    #[test]
    fn zero_height_rows_do_not_hide_the_rows_around_them() {
        // Tops 0 10 10 10 20: the two zero rows sit at 10.
        let layout = uneven(&[10.0, 0.0, 0.0, 10.0]);
        assert_eq!(layout.window(seen(0.0, 10.0), 0), 0..1, "above them");
        assert_eq!(
            layout.window(seen(10.0, 10.0), 0),
            3..4,
            "the row at the offset"
        );
        assert_eq!(
            layout.window(seen(10.0, 10.0), 1),
            2..4,
            "overscan reaches a zero row"
        );
    }

    #[test]
    fn an_even_layout_windows_as_the_scroll_model_does() {
        let even = Layout::Even {
            pitch: Px(20.0),
            len: 100,
        };
        let state = seen(500.0, 100.0);
        assert_eq!(even.window(state, 2), state.rows(Px(20.0), 100, 2));
        assert_eq!(even.total(), Px(2000.0));
        assert_eq!(even.top(7), Px(140.0));
        assert_eq!(even.top(500), Px(2000.0), "clamped to the end");
        assert_eq!(even.height(3), Px(20.0));
        assert_eq!(even.height(100), Px(0.0), "past the end");
    }

    #[test]
    fn a_page_is_the_height_of_the_last_rows() {
        let layout = uneven(&[66.0, 24.0, 66.0, 24.0]);
        // (rows, wanted distance)
        for (rows, want) in [(0, 0.0), (1, 24.0), (2, 90.0), (4, 180.0), (9, 180.0)] {
            assert_eq!(layout.page(rows), Px(want), "{rows} rows");
        }
        let even = Layout::Even {
            pitch: Px(20.0),
            len: 4,
        };
        assert_eq!(even.page(10), Px(200.0));
    }

    #[test]
    fn the_same_heights_are_the_same_layout() {
        let first = uneven(&[1.0, 2.0]);
        let again = Layout::uneven([Px(1.0), Px(2.0)].into_iter(), &first);
        assert_eq!(again, first);
        let other = Layout::uneven([Px(1.0), Px(3.0)].into_iter(), &first);
        assert_ne!(other, first);
    }
}
