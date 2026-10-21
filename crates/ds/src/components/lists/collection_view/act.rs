//! What a press on a tile does: a click reaches its row, a double-click opens it.

use crate::components::controls::press::{button_of, press_of};
use crate::components::lists::reach::Reach;
use crate::focus::press_focus::on_click;
use dioxus::prelude::*;
use ds_core::press::PointerButton;
use ds_core::selection::{Pick, PickHow};
use ds_core::vocab::Selection;

/// The presses a live tile answers, for the row `at`.
#[derive(Clone, Copy)]
pub(crate) struct Act {
    /// The tile's row.
    pub(crate) at: usize,
    /// Where a reached or opened row is reported.
    pub(crate) reach: Reach,
    /// Hears the row a secondary press asked a menu for, with the event.
    pub(crate) menu: Option<EventHandler<(usize, MouseEvent)>>,
    /// Whether the row is selected now.
    pub(crate) selection: Selection,
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

    /// A secondary press (a right click): the tile is selected first unless it already is, so
    /// a menu over a selection of several acts on all of them, then the owner is asked for the
    /// tile's menu. Nothing happens, and the press goes on to the page's own menu, when no one
    /// listens.
    pub(crate) fn context_menu(self, event: &MouseEvent) {
        let Some(menu) = self.menu else {
            return;
        };
        event.prevent_default();
        event.stop_propagation();
        on_click(event);
        if self.selection == Selection::Unselected {
            self.reach.tell(Pick {
                row: self.at,
                how: PickHow::Replace,
            });
        }
        menu.call((self.at, event.clone()));
    }
}
