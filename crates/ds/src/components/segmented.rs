//! SegmentedControl: one choice out of two to four, all visible (design/04-COMPONENTS.md
//! section 3).

use crate::components::vocab::{Selection, Switch};
use dioxus::prelude::*;

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

/// One choice out of a few.
#[component]
pub fn SegmentedControl<T: Clone + PartialEq + 'static>(
    label: String,
    options: Vec<(T, String)>,
    value: T,
    #[props(default)] size: SegSize,
    onchange: EventHandler<T>,
) -> Element {
    rsx! {
        div {
            class: "ds-segmented",
            "data-size": size.slug(),
            role: "group",
            "aria-label": "{label}",
            for (option, text) in options {
                button {
                    r#type: "button",
                    class: "ds-segment",
                    "aria-pressed": pressed(Selection::of(&option, &value)).aria(),
                    onclick: move |_| onchange.call(option.clone()),
                    "{text}"
                }
            }
        }
    }
}
