//! The command palette's selection: its own, kept per query (every keystroke puts it back on
//! the first choice, `S:1651`), or the caller's (sill FINDINGS Q41), and what the caller hears.

use dioxus::core::queue_effect;
use dioxus::prelude::*;

/// Who holds the selection and who hears it, for one render.
#[derive(Clone, PartialEq)]
pub(crate) struct PaletteSelection {
    /// The caller's selection, when it holds it.
    controlled: Option<usize>,
    /// The palette's own, with the query it was made under.
    own: Signal<(String, usize)>,
    /// The last choice reported while the palette held it.
    reported: CopyValue<Option<usize>>,
    on_select: Option<EventHandler<usize>>,
    query: String,
}

/// The palette's selection hook, for this render's props.
pub(crate) fn use_palette_selection(
    query: &str,
    controlled: Option<usize>,
    on_select: Option<EventHandler<usize>>,
) -> PaletteSelection {
    PaletteSelection {
        controlled,
        own: use_signal(|| (query.to_owned(), 0)),
        reported: use_hook(|| CopyValue::new(None)),
        on_select,
        query: query.to_owned(),
    }
}

impl PaletteSelection {
    /// The selected choice among `count`, clamped; the palette's own resets on a new query.
    pub(crate) fn current(&self, count: usize) -> usize {
        let wanted = match (self.controlled, &*self.own.read()) {
            (Some(index), _) => index,
            (None, (made, index)) if *made == self.query => *index,
            (None, _) => 0,
        };
        wanted.min(count.saturating_sub(1))
    }

    /// Tell the caller, after this render, when the palette's own selection changed.
    pub(crate) fn report(&self, current: usize, count: usize) {
        let mut reported = self.reported;
        if self.controlled.is_some() || count == 0 || *reported.peek() == Some(current) {
            return;
        }
        reported.set(Some(current));
        if let Some(on_select) = self.on_select {
            queue_effect(move || on_select.call(current));
        }
    }

    /// Back to the first choice, as a palette shown again starts: the palette's own moves
    /// there and is reported afresh, the caller's is asked for.
    pub(crate) fn reset(&self) {
        let mut reported = self.reported;
        reported.set(None);
        match (self.controlled, self.on_select) {
            (Some(_), Some(on_select)) => on_select.call(0),
            (Some(_), None) => {}
            (None, _) => {
                let mut own = self.own;
                own.set((String::new(), 0));
            }
        }
    }

    /// Move to `next`: the palette's own moves, the caller's is asked for.
    pub(crate) fn select(&self, next: usize) {
        match (self.controlled, self.on_select) {
            (Some(_), Some(on_select)) => on_select.call(next),
            (Some(_), None) => {}
            (None, _) => {
                let mut own = self.own;
                own.set((self.query.clone(), next));
            }
        }
    }
}
