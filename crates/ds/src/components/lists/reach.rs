//! How the owner of a list of pickable rows hears a row being reached, by a click or a key
//! (`VirtualTable`, `CollectionView`).

use chordkit::Platform;
use dioxus::prelude::EventHandler;
use ds_core::press::Press;
use ds_core::selection::{Pick, PickHow};

/// Where a reached row is reported: `onreach` hears the row and how it was reached, else
/// `onselect` hears the row alone.
#[derive(Clone, Copy)]
pub(crate) struct Reach {
    /// The platform whose primary modifier toggles a row.
    pub(crate) platform: Platform,
    /// Hears a `Pick`.
    pub(crate) onreach: Option<EventHandler<Pick>>,
    /// Hears the row, when `onreach` is not given.
    pub(crate) onselect: Option<EventHandler<usize>>,
    /// Hears the row to open (Finder's Open).
    pub(crate) onopen: Option<EventHandler<usize>>,
}

impl Reach {
    /// Whether anyone listens.
    pub(crate) fn heard(self) -> bool {
        self.onreach.is_some() || self.onselect.is_some()
    }

    /// Tell the owner `pick`: `onreach` hears the row and how, else `onselect` hears the row.
    pub(crate) fn tell(self, pick: Pick) {
        match (self.onreach, self.onselect) {
            (Some(onreach), _) => onreach.call(pick),
            (None, Some(onselect)) => onselect.call(pick.row),
            (None, None) => {}
        }
    }

    /// The owner is asked to open `row`, if it listens.
    pub(crate) fn opened(self, row: usize) {
        if let Some(onopen) = self.onopen {
            onopen.call(row);
        }
    }

    /// A click on `row`, with the modifiers it carried.
    pub(crate) fn clicked(self, row: usize, press: Press) {
        let how = PickHow::of_press(self.platform, press.modifiers);
        self.tell(Pick { row, how });
    }
}
