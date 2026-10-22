//! What a rubber-band drag decides, pure: the rectangle between where the drag began and where the
//! pointer is, which rows or tiles it touches, the selection that results and how far the edge of
//! the view scrolls the content under a pointer held near it.

use chordkit::Platform;
use dioxus::prelude::Modifiers;
use ds_core::geometry::units::{Point, Px, Rect, Size};
use ds_core::selection::{PickHow, RowSelection};
use std::ops::Range;

/// Whether a band replaces the selection or adds to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MarqueeHow {
    /// The rows inside the band are the selection.
    Replace,
    /// The rows inside the band are added to the selection the drag began with (Shift, or the
    /// primary modifier: Command on a Mac).
    Add,
}

impl MarqueeHow {
    /// How a drag begun with `modifiers` held selects on `platform`.
    pub(crate) fn of(platform: Platform, modifiers: Modifiers) -> MarqueeHow {
        match PickHow::of_press(platform, modifiers) {
            PickHow::Replace => MarqueeHow::Replace,
            PickHow::Extend | PickHow::Toggle => MarqueeHow::Add,
        }
    }
}

/// The rectangle between two corners, whichever way the pointer went.
pub(crate) fn band(from: Point, to: Point) -> Rect {
    let (left, right) = (from.x.0.min(to.x.0), from.x.0.max(to.x.0));
    let (top, bottom) = (from.y.0.min(to.y.0), from.y.0.max(to.y.0));
    Rect {
        origin: Point {
            x: Px(left),
            y: Px(top),
        },
        size: Size {
            width: Px(right - left),
            height: Px(bottom - top),
        },
    }
}

/// The rows of `len` rows `pitch` tall, from the content's top, that the band touches.
pub(crate) fn rows_in(band: Rect, pitch: Px, len: usize) -> Range<usize> {
    if pitch.0 <= 0.0 || !band.top().0.is_finite() || !band.bottom().0.is_finite() {
        return 0..0;
    }
    let first = (band.top().0.max(0.0) / pitch.0).floor() as usize;
    let end = (band.bottom().0.max(0.0) / pitch.0).ceil() as usize;
    first.min(len)..end.min(len)
}

/// How a grid of tiles is laid out, for finding the tiles a band touches.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct GridShape {
    /// Tiles in a row.
    pub(crate) columns: usize,
    /// The grid's own width, the side padding included.
    pub(crate) width: Px,
    /// The padding on each side of the grid.
    pub(crate) inset: Px,
    /// A row's height.
    pub(crate) pitch: Px,
    /// The widest a tile's own box is (the cell may be wider; the tile sits in its middle).
    pub(crate) tile: Px,
}

impl GridShape {
    /// The columns whose tile the band's horizontal extent touches.
    fn columns_in(self, band: Rect) -> Range<usize> {
        let columns = self.columns.max(1);
        let cell = (self.width.0 - 2.0 * self.inset.0) / columns as f32;
        if cell <= 0.0 {
            return 0..0;
        }
        let hit = cell.min(self.tile.0);
        let margin = (cell - hit) / 2.0;
        let touches = |column: usize| {
            let left = self.inset.0 + column as f32 * cell + margin;
            left < band.right().0 && left + hit > band.left().0
        };
        let first = (0..columns).find(|column| touches(*column));
        let last = (0..columns).rev().find(|column| touches(*column));
        match (first, last) {
            (Some(first), Some(last)) => first..last + 1,
            _ => 0..0,
        }
    }
}

/// The tiles of `len` that the band touches, as one range for each row of tiles.
pub(crate) fn tiles_in(band: Rect, grid: GridShape, len: usize) -> Vec<Range<usize>> {
    let columns = grid.columns.max(1);
    let across = grid.columns_in(band);
    if across.is_empty() {
        return Vec::new();
    }
    rows_in(band, grid.pitch, len.div_ceil(columns))
        .map(|row| {
            let start = row * columns + across.start;
            let end = (row * columns + across.end).min(len);
            start.min(end)..end
        })
        .filter(|range| !range.is_empty())
        .collect()
}

/// The selection a drag makes: the tiles or rows it touches (`hit`), on its own for a `Replace`
/// and added to `base`, the selection the drag began with, for an `Add`.
pub(crate) fn selected(base: &RowSelection, how: MarqueeHow, hit: &[Range<usize>]) -> RowSelection {
    let began = match how {
        MarqueeHow::Replace => RowSelection::none(),
        MarqueeHow::Add => base.clone(),
    };
    began.united(hit)
}

/// How far the edge zone reaches into the view from each side.
const EDGE: f32 = 28.0;
/// The most the content moves for one pointer move at the very edge.
const FASTEST: f32 = 24.0;

/// How far to scroll for the pointer `y` px from the viewport's top in a viewport `height` tall:
/// up (negative) near the top edge, down near the bottom, faster the nearer the edge, and more
/// still when the pointer is past it; zero in the middle. It runs on each pointer move, so a pointer
/// held still does not scroll.
pub(crate) fn edge_scroll(y: f32, height: f32) -> f32 {
    if !(y.is_finite() && height.is_finite()) || height <= 0.0 {
        return 0.0;
    }
    let zone = EDGE.min(height / 2.0);
    let deeper = |into: f32| (into / zone).clamp(0.0, 1.0) * FASTEST;
    if y < zone {
        -deeper(zone - y)
    } else if y > height - zone {
        deeper(y - (height - zone))
    } else {
        0.0
    }
}

#[cfg(test)]
#[allow(clippy::single_range_in_vec_init)] // The tables spell expected selections as lists of ranges.
mod tests {
    use super::{GridShape, MarqueeHow, band, edge_scroll, rows_in, selected, tiles_in};
    use chordkit::Platform;
    use dioxus::prelude::Modifiers;
    use ds_core::geometry::units::{Point, Px, Rect, Size};
    use ds_core::selection::RowSelection;
    use std::ops::Range;

    fn at(x: f32, y: f32) -> Point {
        Point { x: Px(x), y: Px(y) }
    }

    fn rect(left: f32, top: f32, right: f32, bottom: f32) -> Rect {
        Rect {
            origin: at(left, top),
            size: Size {
                width: Px(right - left),
                height: Px(bottom - top),
            },
        }
    }

    #[test]
    fn a_band_is_the_rectangle_between_its_corners_whichever_way_the_pointer_went() {
        let want = rect(10.0, 20.0, 50.0, 90.0);
        assert_eq!(band(at(10.0, 20.0), at(50.0, 90.0)), want);
        assert_eq!(band(at(50.0, 90.0), at(10.0, 20.0)), want);
        assert_eq!(band(at(50.0, 20.0), at(10.0, 90.0)), want);
    }

    #[test]
    fn a_band_touches_the_rows_it_overlaps_and_none_past_the_ends() {
        // (name, top, bottom, rows, wanted)
        let cases: [(&str, f32, f32, usize, Range<usize>); 7] = [
            ("inside one row", 25.0, 30.0, 10, 1..2),
            ("across three", 25.0, 65.0, 10, 1..4),
            ("ends on a row boundary", 20.0, 40.0, 10, 1..2),
            ("above the first row", -50.0, 15.0, 10, 0..1),
            ("past the last row", 80.0, 500.0, 10, 4..10),
            ("wholly below the rows", 300.0, 400.0, 10, 10..10),
            ("no rows", 0.0, 50.0, 0, 0..0),
        ];
        for (name, top, bottom, rows, want) in cases {
            let found = rows_in(rect(0.0, top, 40.0, bottom), Px(20.0), rows);
            assert_eq!(found, want, "{name}");
        }
    }

    fn grid() -> GridShape {
        GridShape {
            columns: 4,
            width: Px(412.0),
            inset: Px(6.0),
            pitch: Px(100.0),
            tile: Px(80.0),
        }
    }

    #[test]
    fn a_band_touches_the_tiles_whose_boxes_it_overlaps_one_range_a_row() {
        // The cell is 100 wide; a tile's box is 80 wide in the middle of its cell, so column 0
        // is 16..96, column 1 116..196, column 2 216..296, column 3 316..396.
        let cases: [(&str, Rect, Vec<Range<usize>>); 6] = [
            ("one tile", rect(20.0, 10.0, 60.0, 50.0), vec![0..1]),
            (
                "two by two",
                rect(150.0, 90.0, 250.0, 150.0),
                vec![1..3, 5..7],
            ),
            (
                "the gap between two tiles touches none",
                rect(97.0, 10.0, 115.0, 50.0),
                vec![],
            ),
            (
                "a short last row stops at the last tile",
                rect(0.0, 200.0, 412.0, 250.0),
                vec![8..10],
            ),
            ("past the rows", rect(0.0, 400.0, 412.0, 500.0), vec![]),
            (
                "the whole width of the first row",
                rect(0.0, 0.0, 412.0, 10.0),
                vec![0..4],
            ),
        ];
        for (name, band, want) in cases {
            assert_eq!(tiles_in(band, grid(), 10), want, "{name}");
        }
    }

    #[test]
    fn a_plain_drag_replaces_and_shift_or_command_adds_to_the_selection_it_began_with() {
        let base = RowSelection::of_rows(Some(1), Some(1), [1, 2]);
        let ranges = |selection: &RowSelection| selection.ranges().to_vec();
        assert_eq!(
            ranges(&selected(&base, MarqueeHow::Replace, &[5..7])),
            vec![5..7]
        );
        assert_eq!(
            ranges(&selected(&base, MarqueeHow::Add, &[5..7])),
            vec![1..3, 5..7]
        );
        assert_eq!(
            ranges(&selected(&base, MarqueeHow::Replace, &[])),
            Vec::<Range<usize>>::new(),
            "a band over nothing clears a replace"
        );
        assert_eq!(
            ranges(&selected(&base, MarqueeHow::Add, &[])),
            vec![1..3],
            "and leaves an add as it was"
        );
    }

    #[test]
    fn shift_and_the_primary_modifier_add_and_anything_else_replaces() {
        let cases = [
            (Modifiers::empty(), MarqueeHow::Replace),
            (Modifiers::SHIFT, MarqueeHow::Add),
            (Modifiers::SUPER, MarqueeHow::Add),
            (Modifiers::ALT, MarqueeHow::Replace),
        ];
        for (held, want) in cases {
            assert_eq!(MarqueeHow::of(Platform::MacOs, held), want, "{held:?}");
        }
    }

    #[test]
    fn the_pointer_near_an_edge_scrolls_toward_it_faster_the_nearer_and_not_in_the_middle() {
        use std::cmp::Ordering::{Equal, Greater, Less};
        // (name, y in the viewport, height, direction, least px, most px)
        let cases = [
            ("middle", 150.0, 300.0, Equal, 0.0, 0.0),
            ("just inside the top zone", 27.0, 300.0, Less, 0.1, 2.0),
            ("on the top edge", 0.0, 300.0, Less, 23.0, 24.1),
            ("above the view", -40.0, 300.0, Less, 23.0, 24.1),
            (
                "just inside the bottom zone",
                273.0,
                300.0,
                Greater,
                0.1,
                2.0,
            ),
            ("below the view", 400.0, 300.0, Greater, 23.0, 24.1),
            ("a view with no height", 5.0, 0.0, Equal, 0.0, 0.0),
        ];
        for (name, y, height, way, least, most) in cases {
            let moved = edge_scroll(y, height);
            assert_eq!(moved.partial_cmp(&0.0), Some(way), "{name}");
            assert!(
                moved.abs() >= least && moved.abs() <= most,
                "{name}: {moved}"
            );
        }
    }
}
