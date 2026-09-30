//! What a menu panel's keys and pointer can rest on: pure, shared by `Menu` and its submenus
//! (design/06-INTERACTIONS.md sections 2.3 and 2.4; design/13-BEHAVIOUR-menus-windows.md section
//! 13.3).
//!
//! A *choice* is an item the selection can rest on: an item or a submenu parent, enabled or not.
//! A header, a status line and a rule are not choices. Choices are numbered in order; the
//! selection, a click and the menu tracker's item path all name a choice by that number. Up and
//! Down skip disabled choices.

use crate::components::menus::item::item::MenuItem;
use ds_core::vocab::Availability;

/// What a choice does when it is picked.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Act<T> {
    /// Yield this value.
    Pick(T),
    /// Open this submenu.
    Open(Vec<MenuItem<T>>),
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
}

/// The choices of `items`, in order: what a choice number names.
pub(crate) fn choices<T: Clone>(items: &[MenuItem<T>]) -> Vec<Choice<T>> {
    items
        .iter()
        .filter_map(|item| match item {
            MenuItem::Item {
                value,
                title,
                availability,
                ..
            } => Some(Choice {
                act: Act::Pick(value.clone()),
                availability: *availability,
                title: title.clone(),
            }),
            MenuItem::Submenu {
                title,
                children,
                availability,
                ..
            } => Some(Choice {
                act: Act::Open(children.clone()),
                availability: *availability,
                title: title.clone(),
            }),
            MenuItem::Header(_) | MenuItem::Info { .. } | MenuItem::Separator => None,
        })
        .collect()
}

/// Each choice's availability, in order.
pub(crate) fn liveness<T>(choices: &[Choice<T>]) -> Vec<Availability> {
    choices.iter().map(|choice| choice.availability).collect()
}
