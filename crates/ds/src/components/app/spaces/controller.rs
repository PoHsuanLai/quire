//! The controller: the Spaces as a signal, switching, editing, and writing.

use super::chord::SwitchChord;
use super::open_menu::OpenMenu;
use dioxus::prelude::*;
use ds_style::space::list::{Refused, Removed, SlideIn, SpaceId, Spaces, Switched};
use ds_style::space::look::SpaceLook;

/// The Spaces of one window, and what drives them. `Copy`: hand it to any handler or view.
///
/// Make one with [`use_spaces`] and pass `handle.look()` to the root's `Ds { look }`, which
/// cross-fades the frame to it over `--t-big`.
pub struct SpacesHandle<P: 'static, R: 'static> {
    pub(super) spaces: Signal<Spaces<P, R>>,
    pub(super) editing: Signal<Option<SpaceId>>,
    pub(super) slide: Signal<Option<SlideIn>>,
    pub(super) menu: Signal<Option<OpenMenu>>,
    pub(super) keep: Callback<Spaces<P, R>>,
    pub(super) here: Callback<(), R>,
    pub(super) arrived: Callback<Switched<R>>,
}

impl<P, R> Clone for SpacesHandle<P, R> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<P, R> Copy for SpacesHandle<P, R> {}

impl<P, R> std::fmt::Debug for SpacesHandle<P, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpacesHandle").finish_non_exhaustive()
    }
}

impl<P, R> PartialEq for SpacesHandle<P, R> {
    fn eq(&self, other: &Self) -> bool {
        self.spaces == other.spaces
    }
}

/// The window's Spaces.
///
/// - `init` builds the list once, at the first render: the stored Spaces
///   (`SpacesStorage::boot_spaces`).
/// - `keep` writes the list whenever a change is final: a switch, a pick, a closed part.
/// - `here` is where the app is now, remembered for the Space being left.
/// - `arrived` shows the place a switch restores, and is where the app slides its content
///   (`handle.slide()`).
pub fn use_spaces<P, R>(
    init: impl FnOnce() -> Spaces<P, R>,
    mut keep: impl FnMut(&Spaces<P, R>) + 'static,
    mut here: impl FnMut() -> R + 'static,
    arrived: impl FnMut(Switched<R>) + 'static,
) -> SpacesHandle<P, R>
where
    P: Clone + PartialEq + 'static,
    R: Clone + Default + PartialEq + 'static,
{
    SpacesHandle {
        spaces: use_signal(init),
        editing: use_signal(|| None),
        slide: use_signal(|| None),
        menu: use_signal(|| None),
        keep: use_callback(move |spaces: Spaces<P, R>| keep(&spaces)),
        here: use_callback(move |()| here()),
        arrived: use_callback(arrived),
    }
}

impl<P: Clone + PartialEq + 'static, R: Clone + Default + PartialEq + 'static> SpacesHandle<P, R> {
    /// The list, as a signal to read in a view.
    pub fn spaces(&self) -> Signal<Spaces<P, R>> {
        self.spaces
    }

    /// The look of the Space on screen: the root's `Ds { look }`.
    pub fn look(&self) -> SpaceLook {
        self.spaces.read().current().look.clone()
    }

    /// The Space on screen.
    pub fn current(&self) -> SpaceId {
        self.spaces.read().current().id
    }

    /// The edge the content slides in from after the last switch, for the app's `data-slide`.
    pub fn slide(&self) -> Option<SlideIn> {
        *self.slide.read()
    }

    /// The Space whose part is open, if any. A switch is refused meanwhile.
    pub fn editing(&self) -> Option<SpaceId> {
        *self.editing.read()
    }

    /// Switch to `id`: remember where the app is, show the Space, write the list, and tell the
    /// app to show the place it was left at. `None` while a part is open, for the current
    /// Space, or for none.
    pub fn switch(&self, id: SpaceId) -> Option<Switched<R>> {
        if self.editing().is_some() {
            return None;
        }
        let here = self.here.call(());
        let switched = self.spaces.write_unchecked().switch_to(id, here)?;
        self.arrive(&switched);
        Some(switched)
    }

    /// [`Self::switch`] to the Space at `index`.
    pub fn switch_index(&self, index: usize) -> Option<Switched<R>> {
        let id = self.spaces.read().at(index)?.id;
        self.switch(id)
    }

    /// A key press on the window: switch when it is `chord` and a digit.
    pub fn on_key(&self, chord: SwitchChord, event: &KeyboardEvent) -> Option<Switched<R>> {
        let number = chord.pressed(&event.key(), event.modifiers())?;
        self.switch_index(usize::from(number.get()) - 1)
    }

    /// "+": add a Space over `payload`, switch to it and write. The app opens its Rename.
    pub fn add(&self, payload: P) -> SpaceId {
        let id = self.spaces.write_unchecked().add(payload);
        if self.switch(id).is_none() {
            self.commit();
        }
        id
    }

    /// Change `id` as it is edited: the frame follows, nothing is written until [`Self::commit`].
    pub fn change(&self, id: SpaceId, edit: impl FnOnce(&mut ds_style::space::list::Space<P>)) {
        self.spaces.write_unchecked().edit(id, edit);
    }

    /// Change where `id` was left, as [`Self::change`] does a Space: nothing is written until
    /// [`Self::commit`].
    pub fn change_recall(&self, id: SpaceId, edit: impl FnOnce(&mut R)) {
        self.spaces.write_unchecked().edit_recall(id, edit);
    }

    /// Write the list.
    pub fn commit(&self) {
        self.keep.call(self.spaces.read().clone());
    }

    /// [`Self::change`] and [`Self::commit`]: one pick.
    pub fn apply(&self, id: SpaceId, edit: impl FnOnce(&mut ds_style::space::list::Space<P>)) {
        self.change(id, edit);
        self.commit();
    }

    /// Delete `id` (never the last) and write the list. When it was the Space on screen, the one
    /// that takes its place is shown as a switch would: `arrived` hears it, sliding in from the
    /// left. The app also calls `TodayHandle::drop_space(id)`.
    pub fn delete(&self, id: SpaceId) -> Result<Removed<P>, Refused> {
        let was = self.current();
        let removed = self.spaces.write_unchecked().remove(id)?;
        if removed.now_current != was {
            let restore = self.spaces.read().recall().of(removed.now_current);
            let mut slide = self.slide;
            slide.set(Some(SlideIn::Left));
            self.arrived.call(Switched {
                from: id,
                to: removed.now_current,
                slide: SlideIn::Left,
                restore,
            });
        }
        self.commit();
        Ok(removed)
    }

    /// Another window wrote the file: take its list, unless a part is being edited, whose
    /// close writes over it.
    pub fn reload(&self, stored: Spaces<P, R>) {
        if self.editing().is_none() && *self.spaces.peek() != stored {
            let mut spaces = self.spaces;
            spaces.set(stored);
        }
    }

    fn arrive(&self, switched: &Switched<R>) {
        let mut slide = self.slide;
        slide.set(Some(switched.slide));
        self.arrived.call(switched.clone());
        self.commit();
    }
}
