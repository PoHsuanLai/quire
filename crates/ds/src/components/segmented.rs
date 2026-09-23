//! SegmentedControl: one choice out of two to four, all visible (design/04-COMPONENTS.md
//! section 3).

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

/// One choice out of a few.
#[component]
pub fn SegmentedControl<T: Clone + PartialEq + 'static>(
    label: String,
    options: Vec<(T, String)>,
    value: T,
    #[props(default)] size: SegSize,
    onchange: EventHandler<T>,
) -> Element {
    todo!()
}
