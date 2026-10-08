//! The Space menu's link rows: follow one of the desktop's Spaces, or stop. Pure.

use super::menu_pick::SpacePick;
use crate::components::menus::item::item::MenuItem;
use ds_core::vocab::{Availability, Check};

/// A Space the desktop has, as the app is told of it: the desktop's own id and the name the
/// person knows it by. The kit never reads inside the id.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DesktopSpace {
    /// The desktop's id for it; stored as a Space's `link`.
    pub id: String,
    /// What the person calls it.
    pub name: String,
}

/// Whether the Space menu offers to link, and what it offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Linking<'a> {
    /// The app has no desktop to link to: the menu has no link rows.
    Off,
    /// The desktop's Spaces, and the one this Space follows now.
    Offered {
        /// The `link` the Space holds.
        current: Option<&'a str>,
        /// Every desktop Space the app was given.
        desktop: &'a [DesktopSpace],
    },
}

/// What a link row asked of the app. The kit has already stored `Linked` and `Unlinked` on the
/// Space; `NewDesktopSpace` waits for the app to make the desktop Space and link it
/// (`SpacesHandle::set_link`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkChange {
    /// The Space now follows the desktop Space with this id.
    Linked(String),
    /// The Space follows none.
    Unlinked,
    /// The person wants a new desktop Space to follow.
    NewDesktopSpace,
}

/// The rows: "Link to..." over the desktop's Spaces and "New Desktop Space...", and "Unlink"
/// while the Space follows one.
pub(super) fn link_rows<A>(linking: Linking<'_>) -> Vec<MenuItem<SpacePick<A>>> {
    let Linking::Offered { current, desktop } = linking else {
        return Vec::new();
    };
    let mut children: Vec<MenuItem<SpacePick<A>>> = desktop
        .iter()
        .map(|one| {
            let check = if current == Some(one.id.as_str()) {
                Check::On
            } else {
                Check::Off
            };
            MenuItem::new(SpacePick::Link(one.id.clone()), one.name.clone()).with_check(check)
        })
        .collect();
    if !children.is_empty() {
        children.push(MenuItem::Separator);
    }
    children.push(MenuItem::new(
        SpacePick::NewDesktopSpace,
        "New Desktop Space\u{2026}",
    ));
    let mut rows = vec![MenuItem::Submenu {
        title: "Link to\u{2026}".to_owned(),
        image: None,
        availability: Availability::Enabled,
        children,
    }];
    if current.is_some() {
        rows.push(MenuItem::new(SpacePick::Unlink, "Unlink"));
    }
    rows
}
