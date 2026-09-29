//! SegmentedControl: one choice out of two to four, all visible (design/04-COMPONENTS.md
//! section 3).
//!
//! The selection is one thumb that slides between equal segments (design/27 section 5.15),
//! driven motion (design/05 section 14): a spring in Rust writes its place, so a second
//! pick mid-slide redirects the thumb from where it is. The thumb stands in the selected
//! segment's own grid cell (`--seg-col`) and is shifted from there by `--seg-dx` segments while
//! it moves, so at rest it covers that segment exactly, whatever the layout rounded the cells
//! to. A label is drawn in the thumb's ink while the thumb is over it (`data-thumb`), not on a
//! timer of its own: the words change colour where the thumb is, never ahead of it or after it.

use crate::core::vocab::{Check, Selection};
use crate::core::word::Word;
use crate::motion::detail::touch::Touch;
use crate::motion::{
    spring_spec::{SpringResponse, SpringSpec},
    timeline::spring::PxPerUnit,
    use_spring::use_spring,
};
use dioxus::prelude::*;

/// About how wide a segment draws, in pixels: what one of the thumb's units is when a hand's
/// speed is scaled into it and when it is close enough to rest.
const SEGMENT_PX: f32 = 64.0;

/// The control's size: `.seg` or the reader's `.view-switch`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum SegSize {
    /// `data-size="regular"`.
    #[default]
    Regular,
    /// `data-size="small"`.
    Small,
}

/// `aria-pressed` for a segment: the pressed fill marks the current choice.
fn pressed(selection: Selection) -> Check {
    match selection {
        Selection::Selected => Check::On,
        Selection::Unselected => Check::Off,
    }
}

/// Closer than this to its cell, in segments, the thumb is drawn in it: a hundredth of a
/// segment is under a pixel.
const IN_CELL: f32 = 0.01;

/// Whether the thumb is over a segment: `data-thumb`, which picks the label's ink.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
enum ThumbOver {
    /// The thumb covers more than half of this segment: the label takes the thumb's ink.
    Under,
    /// The label stands on the track.
    Clear,
}

impl ThumbOver {
    /// Where the thumb, at `position` segments from the start, stands against segment `index`.
    fn of(position: f32, index: usize) -> ThumbOver {
        if (position - index as f32).abs() < 0.5 {
            ThumbOver::Under
        } else {
            ThumbOver::Clear
        }
    }
}

/// The group's inline style: `count` segments, the thumb's home cell (`at`, 1-based for
/// `grid-column`) and how far it stands from it, in segments.
fn group_style(count: usize, at: usize, position: f32) -> String {
    let shift = position - at as f32;
    let shift = if shift.abs() < IN_CELL { 0.0 } else { shift };
    format!("--seg-n:{count};--seg-col:{};--seg-dx:{shift:.3}", at + 1)
}

/// The index of `value` among `options`: where the thumb stands (the first when none matches).
fn index_of<T: PartialEq>(options: &[(T, String)], value: &T) -> usize {
    options
        .iter()
        .position(|(option, _)| option == value)
        .unwrap_or_default()
}

/// One choice out of a few.
#[component]
pub fn SegmentedControl<T: Clone + PartialEq + 'static>(
    label: String,
    options: Vec<(T, String)>,
    value: T,
    #[props(default)] size: SegSize,
    onchange: EventHandler<T>,
) -> Element {
    let at = index_of(&options, &value);
    let count = options.len().max(1);
    let mut asked = use_signal(|| None::<(usize, Touch)>);
    let touch = match *asked.peek() {
        Some((wanted, touch)) if wanted == at => touch,
        Some(_) | None => Touch::Remote,
    };
    let spec = SpringSpec::for_touch(touch).response(SpringResponse::Quick);
    let thumb = use_spring(at as f32, spec, PxPerUnit(SEGMENT_PX));
    rsx! {
        div {
            class: "ds-segmented",
            "data-size": size.slug(),
            role: "group",
            "aria-label": "{label}",
            style: group_style(count, at, thumb.position()),
            for (index , (option , text)) in options.into_iter().enumerate() {
                button {
                    r#type: "button",
                    class: "ds-segment",
                    "aria-pressed": pressed(Selection::of(&option, &value)).aria(),
                    "data-thumb": ThumbOver::of(thumb.position(), index).slug(),
                    onclick: move |event| {
                        asked.set(Some((index, Touch::from_event(&event))));
                        onchange.call(option.clone());
                    },
                    "{text}"
                }
            }
        }
    }
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
        let options = vec![
            (1, "a".to_owned()),
            (2, "b".to_owned()),
            (3, "c".to_owned()),
        ];
        const CASES: &[(i32, usize)] = &[(1, 0), (3, 2), (9, 0)];
        for &(value, want) in CASES {
            assert_eq!(index_of(&options, &value), want, "{value}");
        }
    }
}
