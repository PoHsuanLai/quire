//! What a press on a tile does: a click reaches its row, a double-click opens it.

use crate::components::controls::press::{button_of, press_of};
use crate::components::lists::reach::Reach;
use crate::focus::press_focus::on_click;
use dioxus::prelude::*;
use ds_core::press::PointerButton;

/// The presses a live tile answers, for the row `at`.
#[derive(Clone, Copy)]
pub(crate) struct Act {
    /// The tile's row.
    pub(crate) at: usize,
    /// Where a reached or opened row is reported.
    pub(crate) reach: Reach,
}

impl Act {
    /// A click: a primary press reaches the row with the modifiers it carried (Shift extends, the
    /// primary modifier toggles); the click also takes the keyboard for the grid.
    pub(crate) fn click(self, event: &MouseEvent) {
        on_click(event);
        if button_of(event.trigger_button()) == Some(PointerButton::Primary) {
            self.reach
                .clicked(self.at, press_of(event, PointerButton::Primary));
        }
    }

    /// A double-click opens the row.
    pub(crate) fn open(self) {
        self.reach.opened(self.at);
    }
}
