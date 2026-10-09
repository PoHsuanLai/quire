//! Whether the component that built a set of rows is still mounted.
//!
//! A floating surface (a menu, a pop-up's list, a palette) is drawn by the overlay host, not by the
//! component that owns its handlers. When that component unmounts, dioxus drops every
//! `EventHandler`, `Signal` and `CopyValue` its scope owns at once, but the host takes the surface
//! out of the document a frame later. A pointer event in that frame reaches a row whose handlers
//! are gone, and calling a dropped handler panics (`Dropped(ValueDroppedError)`). Every listener
//! on such a surface asks its [`Alive`] first and does nothing once the owner is gone.

use dioxus::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

/// Whether the owner of a surface's handlers is still mounted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Life {
    /// Mounted: its handlers may be called.
    Up,
    /// Unmounted: its handlers are dropped.
    Gone,
}

/// A shared mark that goes [`Life::Gone`] when the component that made it unmounts. It is plain
/// `Rc` state, not scope-owned, so reading it after the unmount is safe.
#[derive(Debug, Clone)]
pub(crate) struct Alive(Rc<Cell<Life>>);

impl Alive {
    /// Whether the owner is still mounted.
    pub(crate) fn life(&self) -> Life {
        self.0.get()
    }

    /// Run `f` if the owner is still mounted.
    pub(crate) fn run<R>(&self, f: impl FnOnce() -> R) -> Option<R> {
        (self.life() == Life::Up).then(f)
    }
}

/// An [`Alive`] for the calling component, gone when it unmounts.
pub(crate) fn use_alive() -> Alive {
    let alive = use_hook(|| Alive(Rc::new(Cell::new(Life::Up))));
    use_drop({
        let alive = alive.clone();
        move || alive.0.set(Life::Gone)
    });
    alive
}

#[cfg(test)]
mod tests {
    use super::{Alive, Life};
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn a_handler_run_through_a_gone_mark_does_nothing() {
        let mark = Rc::new(Cell::new(Life::Up));
        let alive = Alive(mark.clone());
        assert_eq!(alive.run(|| 1), Some(1));
        mark.set(Life::Gone);
        assert_eq!(alive.run(|| 1), None);
    }
}
