//! SegmentedControl: `NSSegmentedControl` (design/30 section 2.3): a few choices side by side
//! in one well, text or image segments.
//!
//! `SelectOne` selects one segment: the selection is one thumb that slides between equal
//! segments (design/27 section 5.15), driven motion (design/05 section 14): a spring in Rust
//! writes its place, so a second pick mid-slide redirects the thumb from where it is. The thumb
//! stands in the selected segment's own grid cell (`--seg-col`) and is shifted from there by
//! `--seg-dx` segments while it moves, so at rest it covers that segment exactly, whatever the
//! layout rounded the cells to. A label is drawn in the thumb's ink while the thumb is over it
//! (`data-thumb`), not on a timer of its own. `SelectAny` fills each selected segment and
//! `Momentary` fills the one held down; neither has a thumb to slide.
//!
//! Markup: `div.ds-segmented[data-size][data-tracking]` of `button.ds-segmented-segment`
//! (`icon`, `label`), then `span.ds-segmented-indicator`, the thumb, last so the segments keep
//! their child positions.

use crate::components::content::icon_view::IconView;
use crate::components::content::text_runs::text;
use crate::components::controls::choice::Choice;
use crate::components::controls::glyph::glyph_size;
use crate::components::controls::press::{ActivationKeys, activates, disabled, use_pressing};
use crate::components::controls::segmented_thumb::{SEGMENT_PX, ThumbOver, group_style, index_of};
use crate::root::common::Common;
use crate::stack::roving::{Rove, Roving, Wrap};
use dioxus::prelude::*;
use ds_core::vocab::{Availability, Check, Selection};
use ds_core::word::Word;
use ds_motion::detail::touch::Touch;
use ds_motion::{
    spring_spec::{SpringResponse, SpringSpec},
    timeline::spring::PxPerUnit,
    use_spring::use_spring,
};
use ds_style::tokens::control_size::ControlSize;

/// How the segments select (`NSSegmentedControl.SwitchTracking`), with what is selected.
#[derive(Debug, Clone, PartialEq)]
pub enum Tracking<T> {
    /// One segment is selected; picking another moves the selection to it.
    SelectOne(T),
    /// Any number are selected; a pick reports the segment for the caller to add or remove.
    SelectAny(Vec<T>),
    /// None stays selected: a segment is down while it is pressed, and a pick reports it.
    Momentary,
}

/// `data-tracking`: the tracking's word, apart from its payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
enum TrackingWord {
    SelectOne,
    SelectAny,
    Momentary,
}

impl<T> Tracking<T> {
    fn word(&self) -> TrackingWord {
        match self {
            Tracking::SelectOne(_) => TrackingWord::SelectOne,
            Tracking::SelectAny(_) => TrackingWord::SelectAny,
            Tracking::Momentary => TrackingWord::Momentary,
        }
    }
}

impl<T: PartialEq> Tracking<T> {
    /// Whether `value` is selected.
    fn selects(&self, value: &T) -> Selection {
        let selected = match self {
            Tracking::SelectOne(one) => one == value,
            Tracking::SelectAny(any) => any.contains(value),
            Tracking::Momentary => false,
        };
        if selected {
            Selection::Selected
        } else {
            Selection::Unselected
        }
    }
}

/// The segment the arrows and Home and End move to from `at`: the next enabled one, stopping at
/// the ends.
fn rove_to(availability: &[Availability], at: usize, rove: Rove) -> Option<usize> {
    let items = availability.iter().copied().enumerate().collect();
    Roving::new(items, Wrap::Stops)
        .focus(&at)
        .rove(rove)
        .focused()
        .copied()
        .filter(|to| *to != at)
}

/// One choice out of a few, or several, or none held.
#[component]
pub fn SegmentedControl<T: Clone + PartialEq + 'static>(
    label: String,
    choices: Vec<Choice<T>>,
    tracking: Tracking<T>,
    #[props(default)] size: ControlSize,
    #[props(default)] availability: Availability,
    onchange: EventHandler<T>,
    #[props(default)] common: Common,
) -> Element {
    let values: Vec<T> = choices.iter().map(|choice| choice.value.clone()).collect();
    let lives: Vec<Availability> = choices.iter().map(|choice| choice.availability).collect();
    let at = match &tracking {
        Tracking::SelectOne(one) => index_of(&values, one),
        Tracking::SelectAny(_) | Tracking::Momentary => 0,
    };
    let count = choices.len().max(1);
    let mut asked = use_signal(|| None::<(usize, Touch)>);
    let touch = match *asked.peek() {
        Some((wanted, touch)) if wanted == at => touch,
        Some(_) | None => Touch::Remote,
    };
    let spec = SpringSpec::for_touch(touch).response(SpringResponse::Quick);
    let thumb = use_spring(at as f32, spec, PxPerUnit(SEGMENT_PX));
    let one = matches!(tracking, Tracking::SelectOne(_));
    let live = availability == Availability::Enabled;
    let class = common.class("ds-segmented");
    let data = common.data_attributes();
    let word = tracking.word();
    rsx! {
        div {
            id: common.id.clone(),
            class,
            role: if one { "radiogroup" } else { "group" },
            "aria-label": common.aria_label.clone().unwrap_or(label),
            "data-size": size.slug(),
            "data-tracking": word.slug(),
            "data-availability": availability.slug(),
            "aria-disabled": availability.aria_disabled(),
            "aria-busy": availability.aria_busy(),
            style: group_style(count, at, thumb.position()),
            onkeydown: {
                let values = values.clone();
                move |event| {
                    let Some(rove) = Rove::of(&event.key()) else { return };
                    if !one || !live {
                        return;
                    }
                    if let Some(to) = rove_to(&lives, at, rove) {
                        event.prevent_default();
                        asked.set(Some((to, Touch::from_event(&event))));
                        onchange.call(values[to].clone());
                    }
                }
            },
            onmounted: move |event| common.mounted(event),
            ..data,
            for (index , choice) in choices.into_iter().enumerate() {
                Segment {
                    key: "{index}",
                    selected: tracking.selects(&choice.value),
                    thumb: if one { Some(ThumbOver::of(thumb.position(), index)) } else { None },
                    enabled: choice.availability,
                    group: availability,
                    size,
                    onpick: {
                        let picked = choice.value.clone();
                        move |touch: Touch| {
                            if one {
                                asked.set(Some((index, touch)));
                            }
                            onchange.call(picked.clone());
                        }
                    },
                    choice,
                }
            }
            if one {
                span { class: "ds-segmented-indicator", "aria-hidden": "true" }
            }
        }
    }
}

/// One segment: its image and its label. `thumb` says whether the sliding thumb is over it (a
/// `SelectOne` control only); `enabled` is the choice's own availability and `group` the
/// control's: a segment takes input only when both do.
#[component]
fn Segment<T: Clone + PartialEq + 'static>(
    choice: Choice<T>,
    selected: Selection,
    thumb: Option<ThumbOver>,
    enabled: Availability,
    group: Availability,
    size: ControlSize,
    onpick: EventHandler<Touch>,
) -> Element {
    let pressing = use_pressing();
    let live = enabled == Availability::Enabled && group == Availability::Enabled;
    let checked = match selected {
        Selection::Selected => Check::On,
        Selection::Unselected => Check::Off,
    };
    let under = match (thumb, selected) {
        (Some(over), _) => over.slug(),
        (None, Selection::Selected) => ThumbOver::Under.slug(),
        (None, Selection::Unselected) => ThumbOver::Clear.slug(),
    };
    rsx! {
        button {
            r#type: "button",
            class: "ds-segmented-segment",
            role: if thumb.is_some() { "radio" } else { "button" },
            "aria-checked": if thumb.is_some() { Some(checked.aria()) } else { None },
            "aria-pressed": if thumb.is_none() { Some(checked.aria()) } else { None },
            // A `SelectOne` control is one stop in the tab order, its selected segment; the arrows
            // move within it.
            tabindex: match (thumb, selected) {
                (Some(_), Selection::Unselected) => Some("-1"),
                (Some(_), Selection::Selected) | (None, _) => None,
            },
            // An image-only segment says nothing in words, so its name is read from here.
            "aria-label": choice.name.clone(),
            "data-selected": selected.slug(),
            "data-thumb": under,
            "data-availability": enabled.slug(),
            "data-pressed": if live { pressing.attr() } else { None },
            "aria-disabled": enabled.aria_disabled(),
            disabled: disabled(if group == Availability::Enabled { enabled } else { group }),
            onmousedown: move |event| pressing.pointer_down(&event),
            onmouseleave: move |_| pressing.released(),
            onmouseup: move |_| pressing.released(),
            onblur: move |_| pressing.released(),
            onkeyup: move |_| pressing.released(),
            onkeydown: move |event| {
                if live && activates(&event, ActivationKeys::SpaceOnly) {
                    event.prevent_default();
                    event.stop_propagation();
                    pressing.key_down(&event, ActivationKeys::SpaceOnly);
                    onpick.call(Touch::from_event(&event));
                }
            },
            onclick: move |event| {
                if live {
                    onpick.call(Touch::from_event(&event));
                }
            },
            if let Some(source) = choice.icon {
                span { class: "ds-segmented-icon",
                    IconView { source, size: glyph_size(size) }
                }
            }
            span { class: "ds-segmented-label", {text(&choice.label)} }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Tracking, rove_to};
    use crate::stack::roving::{Edge, Rove, Step};
    use ds_core::vocab::Availability::{Disabled as D, Enabled as E};
    use ds_core::vocab::Selection;

    #[test]
    fn the_arrows_skip_a_disabled_segment_and_stop_at_the_ends() {
        // (availability, from, rove, to)
        let cases: &[(&[_], usize, Rove, Option<usize>)] = &[
            (&[E, E, E], 0, Rove::Step(Step::Down), Some(1)),
            (&[E, D, E], 0, Rove::Step(Step::Down), Some(2)),
            (&[E, E, E], 2, Rove::Step(Step::Down), None),
            (&[E, E, E], 0, Rove::Step(Step::Up), None),
            (&[E, E, E], 1, Rove::Edge(Edge::Last), Some(2)),
            (&[E, E, E], 2, Rove::Edge(Edge::Last), None),
        ];
        for &(live, from, rove, want) in cases {
            assert_eq!(rove_to(live, from, rove), want, "{live:?} {from} {rove:?}");
        }
    }

    #[test]
    fn each_tracking_selects_as_it_says() {
        // (tracking, value, selected)
        let cases = [
            (Tracking::SelectOne(1), 1, Selection::Selected),
            (Tracking::SelectOne(1), 2, Selection::Unselected),
            (Tracking::SelectAny(vec![1, 3]), 3, Selection::Selected),
            (Tracking::SelectAny(vec![1, 3]), 2, Selection::Unselected),
            (Tracking::Momentary, 1, Selection::Unselected),
        ];
        for (tracking, value, want) in cases {
            assert_eq!(tracking.selects(&value), want, "{tracking:?} {value}");
        }
    }
}
