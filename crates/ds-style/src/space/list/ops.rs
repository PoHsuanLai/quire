//! Making, renaming, editing and reordering Spaces.

use super::model::{Space, SpaceId, Spaces};
use super::wire::clamp_look;
use crate::space::look::Grain;
use crate::space::presets::{PRESETS, default_look};

impl<P, R> Spaces<P, R> {
    /// A list of one Space, which is on screen.
    pub fn new(first: Space<P>) -> Self {
        Spaces {
            current: first.id,
            next: first.id.0 + 1,
            list: vec![first],
            recall: super::recall::Recall::default(),
        }
    }

    /// A new Space for "+": "Space N", tinted from the next preset in turn, keeping the current
    /// Space's theme so making one does not also change how the window looks. It is appended and
    /// not switched to.
    pub fn add(&mut self, payload: P) -> SpaceId {
        let id = SpaceId(self.next);
        self.next += 1;
        let count = self.list.len();
        let mut look = default_look(count, Grain::default(), self.current().look.card_accent);
        look.dots = PRESETS[count % PRESETS.len()].dots.to_vec();
        look.theme = self.current().look.theme;
        self.list.push(Space {
            id,
            name: format!("Space {}", count + 1),
            link: None,
            look,
            payload,
        });
        id
    }

    /// Give `id` a name. Any text, including none.
    pub fn rename(&mut self, id: SpaceId, name: impl Into<String>) {
        self.edit(id, |space| space.name = name.into());
    }

    /// Link `id` to the desktop Space named `link`, or unlink it with `None`. The kit only
    /// stores the name; a consumer whose desktop Space is gone unlinks it itself.
    pub fn set_link(&mut self, id: SpaceId, link: Option<String>) {
        self.edit(id, |space| space.link = link);
    }

    /// Change `id` in place. Its look is brought back to the dot limits afterwards (one to three
    /// dots, finite hue and chroma), whatever `change` did. Its id is kept.
    pub fn edit(&mut self, id: SpaceId, change: impl FnOnce(&mut Space<P>)) {
        if let Some(space) = self.list.iter_mut().find(|space| space.id == id) {
            change(space);
            space.id = id;
            space.look = clamp_look(std::mem::take(&mut space.look));
        }
    }

    /// Change where `id` was left, from the default place when it has none. A `id` that is not
    /// a Space is ignored.
    pub fn edit_recall(&mut self, id: SpaceId, change: impl FnOnce(&mut R))
    where
        R: Default,
    {
        if self.index_of(id).is_some() {
            self.recall.edit(id, change);
        }
    }

    /// Move `id` so it stands at `index` (past the end is the end). The Space on screen stays.
    pub fn move_to(&mut self, id: SpaceId, index: usize) {
        if let Some(from) = self.index_of(id) {
            let space = self.list.remove(from);
            self.list.insert(index.min(self.list.len()), space);
        }
    }

    /// Show `id` without remembering where the app was: a restore of a saved state, not a switch.
    pub fn select(&mut self, id: SpaceId) {
        if self.index_of(id).is_some() {
            self.current = id;
        }
    }
}
