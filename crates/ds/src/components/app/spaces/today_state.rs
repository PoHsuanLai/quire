//! Today as a signal, written on every change.

use dioxus::prelude::*;
use ds_style::space::list::{Epoch, SpaceId, Today};

/// A window's Today. `Copy`. Closing or expiring an entry never calls into the app's data.
pub struct TodayHandle<I: 'static, K: 'static> {
    pub(super) today: Signal<Today<I, K>>,
    pub(super) keep: Callback<Today<I, K>>,
}

impl<I, K> Clone for TodayHandle<I, K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<I, K> Copy for TodayHandle<I, K> {}

impl<I, K> std::fmt::Debug for TodayHandle<I, K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TodayHandle").finish_non_exhaustive()
    }
}

impl<I, K> PartialEq for TodayHandle<I, K> {
    fn eq(&self, other: &Self) -> bool {
        self.today == other.today
    }
}

/// The window's Today: `init` builds it once (`SpacesStorage::boot_today`, which prunes), `keep`
/// writes it after each change.
pub fn use_today<I, K>(
    init: impl FnOnce() -> Today<I, K>,
    mut keep: impl FnMut(&Today<I, K>) + 'static,
) -> TodayHandle<I, K>
where
    I: Clone + PartialEq + 'static,
    K: Clone + PartialEq + 'static,
{
    TodayHandle {
        today: use_signal(init),
        keep: use_callback(move |today: Today<I, K>| keep(&today)),
    }
}

impl<I: Clone + PartialEq + 'static, K: Clone + PartialEq + 'static> TodayHandle<I, K> {
    /// The state, to read in a view.
    pub fn today(&self) -> Signal<Today<I, K>> {
        self.today
    }

    /// `item` was opened in `space` just now.
    pub fn opened(&self, space: SpaceId, item: I) {
        self.change(|today| today.opened(space, item, Epoch::now()));
    }

    /// Close the shortcut to `item` in `space`.
    pub fn close(&self, space: SpaceId, item: &I) {
        self.change(|today| today.close(space, item));
    }

    /// Close every shortcut in `space`.
    pub fn clear(&self, space: SpaceId) {
        self.change(|today| today.clear(space));
    }

    /// Put `thing` aside in `space` with `title`.
    pub fn park(&self, space: SpaceId, thing: K, title: &str) {
        self.change(|today| today.park(space, thing, title, Epoch::now()));
    }

    /// `thing` is open again, sent or gone.
    pub fn unpark(&self, thing: &K) {
        self.change(|today| today.unpark(thing));
    }

    /// `space` was deleted.
    pub fn drop_space(&self, space: SpaceId) {
        self.change(|today| today.drop_space(space));
    }

    fn change(&self, edit: impl FnOnce(&mut Today<I, K>)) {
        edit(&mut self.today.write_unchecked());
        self.keep.call(self.today.read().clone());
    }
}
