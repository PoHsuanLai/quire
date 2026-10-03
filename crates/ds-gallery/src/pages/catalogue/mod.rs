//! The catalogue page (design/30 section 2): one section per component of the catalogue's controls
//! and fields, every state it can express, so a reviewer sees each beside its neighbours.

mod badge;
mod button;
mod checkbox;
mod key_equivalent;
mod label;
mod level_indicator;
mod progress_indicator;
mod radio_group;
mod scrubber;
mod segmented_control;
mod slider;
mod text_field;
mod toggle;

use dioxus::prelude::*;

/// The Catalogue page.
#[component]
pub fn CataloguePage() -> Element {
    rsx! {
        label::LabelSection {}
        button::ButtonSection {}
        toggle::ToggleSection {}
        checkbox::CheckboxSection {}
        radio_group::RadioGroupSection {}
        segmented_control::SegmentedSection {}
        slider::SliderSection {}
        scrubber::ScrubberSection {}
        text_field::TextFieldSection {}
        progress_indicator::ProgressSection {}
        level_indicator::LevelIndicatorSection {}
        badge::BadgeSection {}
        key_equivalent::KeyEquivalentSection {}
    }
}
