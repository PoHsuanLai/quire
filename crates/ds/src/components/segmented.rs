//! SegmentedControl: one choice out of two to four, all visible (design/04-COMPONENTS.md
//! section 3).
//!
//! The selection is one thumb that slides between equal segments (design/27 section 5.15),
//! driven motion (design/05 section 14, wave H1): a spring in Rust writes its place, `--seg-x`
//! in segments, so a second pick mid-slide redirects the thumb from where it is.

use crate::components::vocab::{Selection, Switch};
use crate::detail::Touch;
use crate::motion::{PxPerUnit, SpringResponse, SpringSpec, use_spring};
use dioxus::prelude::*;

/// About how wide a segment draws, in pixels: what one of the thumb's units is when a hand's
/// speed is scaled into it and when it is close enough to rest.
const SEGMENT_PX: f32 = 64.0;

/// The control's size: `.seg` or the reader's `.view-switch`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegSize {
    /// `data-size="regular"`.
    #[default]
    Regular,
    /// `data-size="small"`.
    Small,
}

impl SegSize {
    /// The `data-size` word.
    fn slug(self) -> &'static str {
        match self {
            SegSize::Regular => "regular",
            SegSize::Small => "small",
        }
    }
}

/// `aria-pressed` for a segment: the pressed fill marks the current choice.
fn pressed(selection: Selection) -> Switch {
    match selection {
        Selection::Selected => Switch::On,
        Selection::Unselected => Switch::Off,
    }
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
            style: "--seg-n:{count};--seg-x:{thumb.css()}",
            span { class: "ds-segment-thumb", "aria-hidden": "true" }
            for (index , (option , text)) in options.into_iter().enumerate() {
                button {
                    r#type: "button",
                    class: "ds-segment",
                    "aria-pressed": pressed(Selection::of(&option, &value)).aria(),
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
    use super::index_of;

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
