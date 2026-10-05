//! Scrolling along the vertical axis, as plain values: where the scroller is, how much it shows and
//! how long its content is, and what follows from that (the offset kept in range, the rows a
//! windowed list must draw, whether the end is near). A host publishes the observed state; a
//! component's model changes it with these functions and asks the host to scroll to the result.

use super::units::Px;
use std::ops::Range;

/// A scroller's state: how far its content is scrolled, how much of it shows, and how long it is.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Scroll {
    /// How far the content is scrolled: the content's top edge sits this far above the
    /// scroller's.
    pub offset: Px,
    /// How much of the content shows at once: the scroller's own height.
    pub viewport: Px,
    /// The content's whole length, scrolled out of view or not.
    pub content: Px,
}

/// An item's extent along the scroll axis, in the scroller's content coordinates (0 is the top of
/// its content at no scroll).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollSpan {
    /// Where it starts.
    pub start: f32,
    /// How long it is.
    pub length: f32,
}

/// The scroll offset that shows `item` in a scrollport `view` long scrolled to `current`, moving
/// the least: unchanged when the item is inside the view (or the view inside it), else the item's
/// nearer edge aligned with the view's (CSSOM `scrollIntoView` with `block: nearest`).
pub fn nearest_scroll(current: f32, view: f32, item: ScrollSpan) -> f32 {
    let end = item.start + item.length;
    let inside = item.start >= current && end <= current + view;
    let covers = item.start <= current && end >= current + view;
    if inside || covers {
        return current;
    }
    let to_start = item.start;
    let to_end = end - view;
    if (to_start - current).abs() < (to_end - current).abs() {
        to_start
    } else {
        to_end
    }
}

impl Scroll {
    /// The largest offset the content allows: none when it fits the viewport.
    pub fn max_offset(self) -> Px {
        Px((self.content.0 - self.viewport.0).max(0.0))
    }

    /// The same scroller with its offset brought into `0..=max_offset`.
    pub fn clamped(self) -> Scroll {
        Scroll {
            offset: Px(self.offset.0.clamp(0.0, self.max_offset().0)),
            ..self
        }
    }

    /// Scrolled to `offset`, kept in range.
    pub fn to(self, offset: Px) -> Scroll {
        Scroll { offset, ..self }.clamped()
    }

    /// Scrolled `delta` further (negative goes back), kept in range.
    pub fn by(self, delta: Px) -> Scroll {
        self.to(self.offset + delta)
    }

    /// Scrolled the least that shows `span`, kept in range.
    pub fn reveal(self, span: ScrollSpan) -> Scroll {
        self.to(Px(nearest_scroll(self.offset.0, self.viewport.0, span)))
    }

    /// Scrolled the least that shows row `index` of rows `pitch` apart.
    pub fn reveal_row(self, index: usize, pitch: Px) -> Scroll {
        self.reveal(ScrollSpan {
            start: index as f32 * pitch.0,
            length: pitch.0,
        })
    }

    /// The rows of `len` rows `pitch` apart that show, plus `overscan` more on each side, as a
    /// range of row indexes. Empty for no rows or a pitch that is not positive; a viewport not
    /// measured yet shows only the overscan below the offset.
    pub fn rows(self, pitch: Px, len: usize, overscan: usize) -> Range<usize> {
        if len == 0 || pitch.0 <= 0.0 {
            return 0..0;
        }
        let first = (self.offset.0.max(0.0) / pitch.0).floor() as usize;
        let past = ((self.offset.0.max(0.0) + self.viewport.0.max(0.0)) / pitch.0).ceil() as usize;
        let start = first.min(len).saturating_sub(overscan);
        let end = past.max(first).saturating_add(overscan).min(len);
        start..end
    }

    /// Whether no more than `within` of the content lies below the viewport: the cue to fetch the
    /// next page. Never before the viewport has been measured.
    pub fn near_end(self, within: Px) -> bool {
        self.viewport.0 > 0.0 && self.content.0 - (self.offset.0 + self.viewport.0) <= within.0
    }
}

#[cfg(test)]
mod tests {
    use super::{Scroll, ScrollSpan, nearest_scroll};
    use crate::geometry::units::Px;

    fn scroll(offset: f32, viewport: f32, content: f32) -> Scroll {
        Scroll {
            offset: Px(offset),
            viewport: Px(viewport),
            content: Px(content),
        }
    }

    #[test]
    fn the_nearest_scroll_moves_the_least_that_shows_the_item() {
        // (current, view, item start, item length, wanted), a 100 long view.
        const CASES: &[(&str, f32, f32, f32, f32, f32)] = &[
            ("inside stays", 0.0, 100.0, 10.0, 20.0, 0.0),
            (
                "flush with both edges stays",
                50.0,
                100.0,
                50.0,
                100.0,
                50.0,
            ),
            ("below aligns its bottom", 0.0, 100.0, 150.0, 20.0, 70.0),
            ("half below aligns its bottom", 0.0, 100.0, 90.0, 20.0, 10.0),
            ("above aligns its top", 200.0, 100.0, 40.0, 20.0, 40.0),
            (
                "half above aligns its top",
                200.0,
                100.0,
                190.0,
                20.0,
                190.0,
            ),
            (
                "taller than the view, covering it, stays",
                100.0,
                100.0,
                50.0,
                300.0,
                100.0,
            ),
            (
                "taller than the view, below, aligns the nearer edge",
                0.0,
                100.0,
                150.0,
                300.0,
                150.0,
            ),
        ];
        for &(name, current, view, start, length, want) in CASES {
            let got = nearest_scroll(current, view, ScrollSpan { start, length });
            assert!((got - want).abs() < 0.001, "{name}: {got}");
        }
    }

    #[test]
    fn the_offset_stays_inside_the_content() {
        // (name, scroller, wanted offset)
        let cases = [
            ("in range", scroll(40.0, 100.0, 500.0), 40.0),
            ("negative", scroll(-30.0, 100.0, 500.0), 0.0),
            ("past the end", scroll(900.0, 100.0, 500.0), 400.0),
            (
                "content shorter than the view",
                scroll(25.0, 100.0, 60.0),
                0.0,
            ),
            ("nothing measured", scroll(25.0, 0.0, 0.0), 0.0),
        ];
        for (name, state, want) in cases {
            assert_eq!(state.clamped().offset, Px(want), "{name}");
        }
    }

    #[test]
    fn to_and_by_move_the_offset_and_keep_it_in_range() {
        let state = scroll(100.0, 100.0, 500.0);
        let cases = [
            ("to", state.to(Px(250.0)), 250.0),
            ("to past the end", state.to(Px(9000.0)), 400.0),
            ("to before the start", state.to(Px(-5.0)), 0.0),
            ("by forward", state.by(Px(60.0)), 160.0),
            ("by back", state.by(Px(-60.0)), 40.0),
            ("by back past the start", state.by(Px(-600.0)), 0.0),
            ("by forward past the end", state.by(Px(600.0)), 400.0),
        ];
        for (name, got, want) in cases {
            assert_eq!(got.offset, Px(want), "{name}");
            assert_eq!(got.viewport, state.viewport, "{name} keeps the viewport");
            assert_eq!(got.content, state.content, "{name} keeps the content");
        }
    }

    #[test]
    fn revealing_a_row_moves_the_least() {
        // 20 px rows in a 100 px viewport over 100 rows.
        let at = |offset| scroll(offset, 100.0, 2000.0);
        let pitch = Px(20.0);
        let cases = [
            ("already shown", at(0.0), 3, 0.0),
            ("last shown row", at(0.0), 4, 0.0),
            ("one row below", at(0.0), 5, 20.0),
            ("far below", at(0.0), 50, 920.0),
            ("above", at(500.0), 10, 200.0),
            ("the last row", at(0.0), 99, 1900.0),
        ];
        for (name, state, index, want) in cases {
            assert_eq!(state.reveal_row(index, pitch).offset, Px(want), "{name}");
        }
    }

    #[test]
    fn the_rows_that_show_are_the_viewport_plus_overscan() {
        // (name, scroller, pitch, rows, overscan, wanted rows)
        let cases = [
            ("top", scroll(0.0, 100.0, 2000.0), 20.0, 100, 0, 0..5),
            (
                "top with overscan",
                scroll(0.0, 100.0, 2000.0),
                20.0,
                100,
                2,
                0..7,
            ),
            (
                "scrolled to a row edge",
                scroll(200.0, 100.0, 2000.0),
                20.0,
                100,
                0,
                10..15,
            ),
            (
                "scrolled half a row",
                scroll(210.0, 100.0, 2000.0),
                20.0,
                100,
                0,
                10..16,
            ),
            (
                "overscan both sides",
                scroll(200.0, 100.0, 2000.0),
                20.0,
                100,
                3,
                7..18,
            ),
            (
                "bottom",
                scroll(1900.0, 100.0, 2000.0),
                20.0,
                100,
                2,
                93..100,
            ),
            (
                "fewer rows than the view",
                scroll(0.0, 100.0, 60.0),
                20.0,
                3,
                2,
                0..3,
            ),
            ("no rows", scroll(0.0, 100.0, 0.0), 20.0, 0, 2, 0..0),
            (
                "viewport not measured",
                scroll(0.0, 0.0, 0.0),
                20.0,
                100,
                2,
                0..2,
            ),
            (
                "pitch not positive",
                scroll(0.0, 100.0, 2000.0),
                0.0,
                100,
                2,
                0..0,
            ),
            (
                "stale offset past the rows",
                scroll(5000.0, 100.0, 2000.0),
                20.0,
                100,
                2,
                98..100,
            ),
        ];
        for (name, state, pitch, len, overscan, want) in cases {
            assert_eq!(state.rows(Px(pitch), len, overscan), want, "{name}");
        }
    }

    #[test]
    fn the_end_is_near_when_little_content_lies_below() {
        // (name, scroller, within, wanted)
        let cases = [
            (
                "top of a long list",
                scroll(0.0, 100.0, 2000.0),
                300.0,
                false,
            ),
            (
                "just outside the margin",
                scroll(1599.0, 100.0, 2000.0),
                300.0,
                false,
            ),
            ("on the margin", scroll(1600.0, 100.0, 2000.0), 300.0, true),
            ("at the end", scroll(1900.0, 100.0, 2000.0), 300.0, true),
            ("content fits the view", scroll(0.0, 100.0, 60.0), 0.0, true),
            ("empty", scroll(0.0, 100.0, 0.0), 0.0, true),
            ("viewport not measured", scroll(0.0, 0.0, 0.0), 300.0, false),
        ];
        for (name, state, within, want) in cases {
            assert_eq!(state.near_end(Px(within)), want, "{name}");
        }
    }
}
