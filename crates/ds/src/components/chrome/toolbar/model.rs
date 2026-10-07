//! What a `Toolbar` shows: its items as data, and the pure rule of how many of them fit before
//! the rest go behind the overflow chevron.

use crate::host::measure::Anchor;
use ds_core::geometry::units::Px;
use ds_core::vocab::{Availability, Check};
use ds_style::icon::Icon;

/// How wide one toolbar button and its gap are, in pixels (`--btn-h` Large plus the gap).
pub const ITEM_PITCH: Px = Px(40.0);

/// What the title keeps for itself, in pixels: items give way before the title does.
pub const TITLE_ROOM: Px = Px(140.0);

/// The padding at each end of the band, in pixels.
const BAND_PADDING: Px = Px(12.0);

/// The gap between the band's items, in pixels.
const ITEM_GAP: Px = Px(8.0);

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

/// What picking a toolbar item reports: its value, and the button it was picked from, so a menu
/// or a popover can hang from that button.
#[derive(Debug, Clone, PartialEq)]
pub struct Picked<T> {
    /// The item's value.
    pub value: T,
    /// The button the pick came from (the overflow chevron for an item that was behind it);
    /// `None` until the button has mounted, which a click never precedes on a real document.
    pub anchor: Option<Anchor>,
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

/// Whether the band's search field has room to stay a field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchFit {
    /// Every item and the field at its minimum width fit.
    Inline,
    /// They do not: the field collapses to a magnifier button.
    Collapsed,
}

/// Whether a search field of at least `min_width` and every item fit in `room`. The field gives
/// way before any item goes behind the chevron, so it collapses as soon as keeping it would push
/// one off.
pub fn search_fit(leading: usize, trailing: usize, room: Px, min_width: Px) -> SearchFit {
    let field = Px(min_width.0 + ITEM_GAP.0);
    if Kept::fitting(leading, trailing, Px(room.0 - field.0)).all(leading, trailing) {
        SearchFit::Inline
    } else {
        SearchFit::Collapsed
    }
}

/// Whether a collapsed search field has been opened by pressing its magnifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Expansion {
    /// The magnifier button alone.
    #[default]
    Folded,
    /// The field, where the button was.
    Open,
}

/// The room the search item takes in the band: the field and its gap, or one button while it is
/// collapsed and folded.
pub(crate) fn search_room(fit: SearchFit, expansion: Expansion, min_width: Px) -> Px {
    match (fit, expansion) {
        (SearchFit::Collapsed, Expansion::Folded) => ITEM_PITCH,
        (SearchFit::Inline, _) | (SearchFit::Collapsed, Expansion::Open) => {
            Px(min_width.0 + ITEM_GAP.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Expansion, Kept, SearchFit, search_fit, search_room};
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

    #[test]
    fn the_search_field_collapses_before_an_item_goes_behind_the_chevron() {
        // Band padding 24, title 140, each item 40, the field its minimum plus a gap of 8.
        /// name, room, leading items, trailing items, field minimum, fit.
        type Case = (&'static str, f32, usize, usize, f32, SearchFit);
        const CASES: &[Case] = &[
            ("plenty of room", 900.0, 1, 2, 200.0, SearchFit::Inline),
            ("exactly enough", 492.0, 1, 2, 200.0, SearchFit::Inline),
            ("a pixel short", 491.0, 1, 2, 200.0, SearchFit::Collapsed),
            (
                "no items: only the field and the title",
                372.0,
                0,
                0,
                200.0,
                SearchFit::Inline,
            ),
            (
                "a wider minimum collapses sooner",
                532.0,
                1,
                2,
                260.0,
                SearchFit::Collapsed,
            ),
        ];
        for &(name, room, leading, trailing, min, want) in CASES {
            assert_eq!(
                search_fit(leading, trailing, Px(room), Px(min)),
                want,
                "{name}"
            );
        }
    }

    #[test]
    fn a_folded_search_item_is_one_button_and_an_open_one_is_the_field() {
        // (fit, expansion, room)
        const CASES: &[(SearchFit, Expansion, f32)] = &[
            (SearchFit::Inline, Expansion::Folded, 208.0),
            (SearchFit::Collapsed, Expansion::Folded, 40.0),
            (SearchFit::Collapsed, Expansion::Open, 208.0),
        ];
        for &(fit, expansion, room) in CASES {
            assert_eq!(
                search_room(fit, expansion, Px(200.0)),
                Px(room),
                "{fit:?} {expansion:?}"
            );
        }
    }
}
