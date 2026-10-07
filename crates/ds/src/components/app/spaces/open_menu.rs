//! Which Space's menu, or one of its parts, is open and where.

use super::controller::SpacesHandle;
use dioxus::prelude::*;
use ds_core::geometry::units::Point;
use ds_style::space::list::SpaceId;

/// What is showing for a Space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Showing {
    /// The menu.
    Menu,
    /// The name field.
    Rename,
    /// The colour field, stops, grain and presets.
    Colour,
    /// The delete question.
    Delete,
}

/// A Space's menu or part, open at a point in the window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OpenMenu {
    /// Whose.
    pub space: SpaceId,
    /// The pointer when it opened, in the window's client pixels.
    pub at: Point,
    /// Which part.
    pub showing: Showing,
}

impl<P: Clone + PartialEq + 'static, R: Clone + Default + PartialEq + 'static> SpacesHandle<P, R> {
    /// What is open, if anything.
    pub fn open(&self) -> Option<OpenMenu> {
        *self.menu.read()
    }

    /// Open `space`'s menu at `at` (a right click on its name or dot).
    pub fn open_menu(&self, space: SpaceId, at: Point) {
        self.show(space, at, Showing::Menu);
    }

    /// Open one of `space`'s parts at `at`. Colour, Rename and Delete hold the Space as being
    /// edited, so a dot click does not switch away and a reload waits, until [`Self::close_menu`].
    pub fn show(&self, space: SpaceId, at: Point, showing: Showing) {
        let (mut menu, mut editing) = (self.menu, self.editing);
        match showing {
            Showing::Menu => editing.set(None),
            Showing::Rename | Showing::Colour | Showing::Delete => editing.set(Some(space)),
        }
        menu.set(Some(OpenMenu { space, at, showing }));
    }

    /// Close the menu or the part, keeping what a part changed.
    pub fn close_menu(&self) {
        let (mut menu, mut editing) = (self.menu, self.editing);
        if editing.peek().is_some() {
            editing.set(None);
            self.commit();
        }
        if menu.peek().is_some() {
            menu.set(None);
        }
    }
}
