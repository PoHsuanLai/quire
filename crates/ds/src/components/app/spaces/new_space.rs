//! Whether the foot draws its own New Space button.

/// Whether [`super::SpacesFoot`] draws the `+` that adds a Space.
///
/// An app whose sidebar foot gathers its commands in one menu (Dia's chevron) hides it and puts
/// New Space in that menu: it names the Space with `SpacesHandle::begin_new(payload, at)`, as the
/// button would, and the Space is added once it is named.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NewSpace {
    /// The foot draws the `+` after the dots.
    #[default]
    Shown,
    /// The foot leaves it out; the app makes a Space from its own menu.
    Hidden,
}
