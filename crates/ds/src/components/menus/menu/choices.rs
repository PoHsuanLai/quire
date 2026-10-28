//! What a menu panel's keys and pointer can rest on: pure, shared by `Menu` and its submenus
//! (design/06-INTERACTIONS.md sections 2.3 and 2.4; design/13-BEHAVIOUR-menus-windows.md section
//! 13.3).
//!
//! A *choice* is an item the selection can rest on: an item or a submenu parent, enabled or not.
//! A header, a status line and a rule are not choices. Choices are numbered in order; the
//! selection, a click and the menu tracker's item path all name a choice by that number. Up and
//! Down skip disabled choices.

use crate::components::menus::item::item::{AfterPick, MenuItem};
use ds_core::vocab::Availability;

/// What a choice does when it is picked.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Act<T> {
    /// Yield this value, then close the menu or leave it open.
    Pick(T, AfterPick),
    /// Open this submenu.
    Open(Vec<MenuItem<T>>),
}

/// Where a choice stands in its row: alone, or one dot of a [`MenuItem::Swatches`] row, whose
/// dots are consecutive choices. Up and Down take the row as one stop; Left and Right move
/// across its dots.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Slot {
    /// The choice is its own row.
    Alone,
    /// The `pos`th of `len` dots.
    Dot { pos: usize, len: usize },
}

/// One choice: what it does, and whether it can.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Choice<T> {
    /// Pick or open.
    pub act: Act<T>,
    /// Enabled or disabled.
    pub availability: Availability,
    /// What the choice is called: what typing a letter matches.
    pub title: String,
    /// Alone, or one dot of a row of them.
    pub slot: Slot,
}

/// The choices of `items`, in order: what a choice number names. A row of swatches is one choice
/// for each of its dots.
pub(crate) fn choices<T: Clone>(items: &[MenuItem<T>]) -> Vec<Choice<T>> {
    items.iter().flat_map(choices_of).collect()
}

/// The choices one item stands for: none for a header, status line or rule.
fn choices_of<T: Clone>(item: &MenuItem<T>) -> Vec<Choice<T>> {
    match item {
        MenuItem::Item {
            value,
            title,
            availability,
            after,
            ..
        } => vec![Choice {
            act: Act::Pick(value.clone(), *after),
            availability: *availability,
            title: title.clone(),
            slot: Slot::Alone,
        }],
        MenuItem::Submenu {
            title,
            children,
            availability,
            ..
        } => vec![Choice {
            act: Act::Open(children.clone()),
            availability: *availability,
            title: title.clone(),
            slot: Slot::Alone,
        }],
        MenuItem::Swatches(row) => row
            .swatches
            .iter()
            .enumerate()
            .map(|(pos, dot)| Choice {
                act: Act::Pick(dot.value.clone(), dot.after),
                availability: row.availability,
                title: dot.name.clone(),
                slot: Slot::Dot {
                    pos,
                    len: row.swatches.len(),
                },
            })
            .collect(),
        MenuItem::Header(_) | MenuItem::Info { .. } | MenuItem::Separator => Vec::new(),
    }
}

/// Each choice's availability, in order.
pub(crate) fn liveness<T>(choices: &[Choice<T>]) -> Vec<Availability> {
    choices.iter().map(|choice| choice.availability).collect()
}

#[cfg(test)]
mod tests {
    use super::{Act, Slot, choices};
    use crate::components::menus::item::item::{MenuItem, MenuSwatch};
    use ds_style::tokens::system_hue::SystemHue;

    #[test]
    fn a_row_of_swatches_is_one_choice_for_each_dot() {
        let items = vec![
            MenuItem::new(0u8, "Open"),
            MenuItem::Separator,
            MenuItem::swatches(
                "Tags",
                vec![
                    MenuSwatch::new(1, "Red", SystemHue::Red),
                    MenuSwatch::new(2, "Blue", SystemHue::Blue),
                ],
            ),
            MenuItem::new(3u8, "Delete"),
        ];
        let got: Vec<(String, Slot)> = choices(&items)
            .into_iter()
            .map(|choice| (choice.title, choice.slot))
            .collect();
        let want = [
            ("Open", Slot::Alone),
            ("Red", Slot::Dot { pos: 0, len: 2 }),
            ("Blue", Slot::Dot { pos: 1, len: 2 }),
            ("Delete", Slot::Alone),
        ];
        assert_eq!(got.len(), want.len());
        for ((title, slot), (want_title, want_slot)) in got.iter().zip(want) {
            assert_eq!((title.as_str(), *slot), (want_title, want_slot));
        }
        assert!(matches!(choices(&items)[2].act, Act::Pick(2, _)));
    }
}
