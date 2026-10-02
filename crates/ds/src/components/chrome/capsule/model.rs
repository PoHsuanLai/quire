//! What a `Capsule` shows: its parts as data, left to right.

use crate::components::chrome::toolbar::model::ToolbarItem;
use crate::components::controls::scrubber_model::BufferedRange;
use ds_core::vocab::{Availability, Fraction};
use ds_motion::spring::Millis;

/// A recording's progress bar in a capsule: it fills the width the buttons leave.
#[derive(Debug, Clone, PartialEq)]
pub struct ScrubSlot {
    /// What a screen reader calls it.
    pub label: String,
    /// How far through the recording it is, in thousandths.
    pub position: Fraction,
    /// How long the recording runs, for the tooltip's time.
    pub length: Millis,
    /// The stretches already loaded.
    pub buffered: Vec<BufferedRange>,
    /// Whether it takes input.
    pub availability: Availability,
}

/// A level (volume) in a capsule: a fixed-width slider with its speaker glyph.
#[derive(Debug, Clone, PartialEq)]
pub struct LevelSlot {
    /// What a screen reader calls it.
    pub label: String,
    /// The level, in thousandths.
    pub value: Fraction,
    /// Whether it takes input.
    pub availability: Availability,
}

/// What the capsule's scrubber reports, as the `Scrubber`'s own handlers do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrubEvent {
    /// A press began at this place.
    Start(Fraction),
    /// A drag moved to this place.
    Move(Fraction),
    /// The press ended at this place.
    End(Fraction),
    /// The drag was abandoned.
    Cancel,
    /// A key asks for this place.
    Seek(Fraction),
}

/// One part of a capsule.
#[derive(Debug, Clone, PartialEq)]
pub enum CapsuleSlot<T> {
    /// A toolbar button, or a toggle.
    Item(ToolbarItem<T>),
    /// A value shown between the buttons (a zoom level, a page number, an elapsed time), in
    /// tabular figures so the capsule does not change width as it counts.
    Readout(String),
    /// A hairline between two groups of buttons.
    Divider,
    /// A progress bar that fills the width the other slots leave (`onscrub` hears it).
    Scrub(ScrubSlot),
    /// A fixed-width level (`onlevel` hears it).
    Level(LevelSlot),
}

impl<T> CapsuleSlot<T> {
    /// The slot of a plain, enabled button.
    pub fn button(value: T, label: impl Into<String>, icon: ds_style::icon::Icon) -> Self {
        CapsuleSlot::Item(ToolbarItem::new(value, label, icon))
    }
}

/// Which of `slots` are buttons a person can press, in order: the capsule's tab stops.
pub fn pressable<T>(slots: &[CapsuleSlot<T>]) -> impl Iterator<Item = &ToolbarItem<T>> {
    slots.iter().filter_map(|slot| match slot {
        CapsuleSlot::Item(item) => Some(item),
        CapsuleSlot::Readout(_)
        | CapsuleSlot::Divider
        | CapsuleSlot::Scrub(_)
        | CapsuleSlot::Level(_) => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ds_style::icon::Icon;

    #[test]
    fn only_items_are_pressable() {
        let slots = vec![
            CapsuleSlot::button(1, "Zoom out", Icon::Minus),
            CapsuleSlot::Readout("100%".to_owned()),
            CapsuleSlot::button(2, "Zoom in", Icon::Plus),
            CapsuleSlot::Divider,
            CapsuleSlot::button(3, "Rotate", Icon::Refresh),
            CapsuleSlot::Scrub(ScrubSlot {
                label: "Position".to_owned(),
                position: Fraction(0),
                length: Millis(1000),
                buffered: Vec::new(),
                availability: Availability::Enabled,
            }),
            CapsuleSlot::Level(LevelSlot {
                label: "Volume".to_owned(),
                value: Fraction(500),
                availability: Availability::Enabled,
            }),
        ];
        let values: Vec<i32> = pressable(&slots).map(|item| item.value).collect();
        assert_eq!(values, vec![1, 2, 3]);
    }
}
