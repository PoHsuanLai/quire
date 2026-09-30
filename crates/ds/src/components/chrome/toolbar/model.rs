//! What a `Toolbar` shows: its items as data, and the pure rule of how many of them fit before
//! the rest go behind the overflow chevron.

use ds_core::geometry::units::Px;
use ds_core::vocab::{Availability, Check};
use ds_style::icon::Icon;

/// How wide one toolbar button and its gap are, in pixels (`--btn-h` Large plus the gap).
pub const ITEM_PITCH: Px = Px(40.0);

/// What the title keeps for itself, in pixels: items give way before the title does.
pub const TITLE_ROOM: Px = Px(140.0);

/// The padding at each end of the band, in pixels.
const BAND_PADDING: Px = Px(12.0);

/// One button of a toolbar: an image and the name a tooltip, a screen reader and the overflow
/// menu give it.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolbarItem<T> {
    /// What picking it reports.
    pub value: T,
    /// Its name.
    pub label: String,
    /// Its image.
    pub icon: Icon,
    /// A toggle's state; `None` for a plain button.
    pub check: Option<Check>,
    /// Whether it can be picked.
    pub availability: Availability,
}

impl<T> ToolbarItem<T> {
    /// A plain, enabled button.
    pub fn new(value: T, label: impl Into<String>, icon: Icon) -> Self {
        ToolbarItem {
            value,
            label: label.into(),
            icon,
            check: None,
            availability: Availability::Enabled,
        }
    }

    /// The same item as a toggle in state `check`.
    pub fn toggle(self, check: Check) -> Self {
        ToolbarItem {
            check: Some(check),
            ..self
        }
    }

    /// The same item with `availability`.
    pub fn with(self, availability: Availability) -> Self {
        ToolbarItem {
            availability,
            ..self
        }
    }
}

/// Where the toolbar gets the width it lays its items out in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToolbarRoom {
    /// Its own width, read once the band has been laid out.
    Measured,
    /// This width: a posed toolbar, or one whose window says how wide it is.
    Fixed(Px),
}

/// How many of the leading and of the trailing items stay on the band.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Kept {
    /// The leading items kept, from the first.
    pub leading: usize,
    /// The trailing items kept, from the first.
    pub trailing: usize,
}

impl Kept {
    /// How many of `leading` and `trailing` items fit in `room`. Items go behind the chevron from
    /// the last trailing one backwards, and the chevron takes an item's place once anything has.
    pub fn fitting(leading: usize, trailing: usize, room: Px) -> Kept {
        let available = room.0 - TITLE_ROOM.0 - 2.0 * BAND_PADDING.0;
        let total = leading + trailing;
        let kept = if (total as f32) * ITEM_PITCH.0 <= available {
            total
        } else {
            (((available - ITEM_PITCH.0) / ITEM_PITCH.0).floor().max(0.0)) as usize
        };
        Kept {
            leading: kept.min(leading),
            trailing: kept.saturating_sub(leading).min(trailing),
        }
    }

    /// Whether every item is on the band.
    pub fn all(self, leading: usize, trailing: usize) -> bool {
        self.leading == leading && self.trailing == trailing
    }
}

#[cfg(test)]
mod tests {
    use super::Kept;
    use ds_core::geometry::units::Px;

    #[test]
    fn items_give_way_from_the_last_trailing_one() {
        // Band padding 24, title 140: each item takes 40, and the chevron one item's place.
        /// name, room, leading items, trailing items, kept leading and trailing.
        type Case = (&'static str, f32, usize, usize, (usize, usize));
        const CASES: &[Case] = &[
            ("plenty of room", 900.0, 2, 3, (2, 3)),
            ("exactly enough", 324.0, 1, 3, (1, 3)),
            ("one short: the chevron takes a place", 323.0, 1, 3, (1, 1)),
            ("into the leading items", 284.0, 2, 3, (2, 0)),
            ("no room at all", 100.0, 2, 3, (0, 0)),
        ];
        for &(name, room, leading, trailing, (kept_leading, kept_trailing)) in CASES {
            assert_eq!(
                Kept::fitting(leading, trailing, Px(room)),
                Kept {
                    leading: kept_leading,
                    trailing: kept_trailing
                },
                "{name}"
            );
        }
    }
}
