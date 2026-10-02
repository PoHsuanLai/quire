//! What a `Capsule` shows: its parts as data, left to right.

use crate::components::chrome::toolbar::model::ToolbarItem;

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
        CapsuleSlot::Readout(_) | CapsuleSlot::Divider => None,
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
        ];
        let values: Vec<i32> = pressable(&slots).map(|item| item.value).collect();
        assert_eq!(values, vec![1, 2, 3]);
    }
}
