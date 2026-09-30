//! The pure half of the segmented control's selection thumb (design/30 section 2.3): where the
//! thumb stands, which labels it covers, and the inline style that places it.

use ds_core::word::Word;

/// About how wide a segment draws, in pixels: what one of the thumb's units is when a hand's
/// speed is scaled into it and when it is close enough to rest.
pub(crate) const SEGMENT_PX: f32 = 64.0;

/// Closer than this to its cell, in segments, the thumb is drawn in it: a hundredth of a
/// segment is under a pixel.
const IN_CELL: f32 = 0.01;

/// Whether the thumb is over a segment: `data-thumb`, which picks the label's ink.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub(crate) enum ThumbOver {
    /// The thumb covers more than half of this segment: the label takes the thumb's ink.
    Under,
    /// The label stands on the track.
    Clear,
}

impl ThumbOver {
    /// Where the thumb, at `position` segments from the start, stands against segment `index`.
    pub(crate) fn of(position: f32, index: usize) -> ThumbOver {
        if (position - index as f32).abs() < 0.5 {
            ThumbOver::Under
        } else {
            ThumbOver::Clear
        }
    }
}

/// The group's inline style: `count` segments, the thumb's home cell (`at`, 1-based for
/// `grid-column`) and how far it stands from it, in segments.
pub(crate) fn group_style(count: usize, at: usize, position: f32) -> String {
    let shift = position - at as f32;
    let shift = if shift.abs() < IN_CELL { 0.0 } else { shift };
    format!("--seg-n:{count};--seg-col:{};--seg-dx:{shift:.3}", at + 1)
}

/// The index of `value` among `values`: where the thumb stands (the first when none matches).
pub(crate) fn index_of<T: PartialEq>(values: &[T], value: &T) -> usize {
    values
        .iter()
        .position(|option| option == value)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{ThumbOver, group_style, index_of};

    #[test]
    fn a_label_takes_the_thumbs_ink_only_while_the_thumb_is_over_it() {
        const CASES: &[(f32, usize, ThumbOver)] = &[
            (0.0, 0, ThumbOver::Under),
            (0.0, 1, ThumbOver::Clear),
            (0.49, 0, ThumbOver::Under),
            (0.51, 0, ThumbOver::Clear),
            (0.51, 1, ThumbOver::Under),
            (1.2, 1, ThumbOver::Under),
            (1.2, 2, ThumbOver::Clear),
            // An overshoot past the last segment still covers it.
            (2.3, 2, ThumbOver::Under),
        ];
        for &(position, index, want) in CASES {
            assert_eq!(ThumbOver::of(position, index), want, "{position} {index}");
        }
    }

    #[test]
    fn the_thumb_stands_in_its_cell_and_is_shifted_while_it_moves() {
        const CASES: &[(usize, f32, &str)] = &[
            (0, 0.0, "--seg-n:3;--seg-col:1;--seg-dx:0.000"),
            (2, 0.0, "--seg-n:3;--seg-col:3;--seg-dx:-2.000"),
            (2, 1.25, "--seg-n:3;--seg-col:3;--seg-dx:-0.750"),
            // Within a hundredth of its cell it is in it: no sub-pixel shift at rest.
            (1, 1.004, "--seg-n:3;--seg-col:2;--seg-dx:0.000"),
        ];
        for &(at, position, want) in CASES {
            assert_eq!(group_style(3, at, position), want, "{at} {position}");
        }
    }

    #[test]
    fn the_thumb_stands_on_the_value_or_the_first() {
        let values = [1, 2, 3];
        const CASES: &[(i32, usize)] = &[(1, 0), (3, 2), (9, 0)];
        for &(value, want) in CASES {
            assert_eq!(index_of(&values, &value), want, "{value}");
        }
    }
}
