//! What the Spaces bar is given: the Spaces it lists and how much of it shows.

use crate::components::content::image_source::ImageSource;
use ds_core::vocab::{DropState, Selection};
use ds_core::word::Word;

/// Which Space a press names: the host's own id for it, quoted back unchanged. The bar never
/// reads it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BarSpaceKey(pub String);

/// One Space on the bar.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SpaceTile {
    /// Which Space it is.
    pub key: BarSpaceKey,
    /// Its name, shown in the collapsed bar and under its thumbnail.
    pub name: String,
    /// Whether it is the Space on show (`aria-current`).
    pub current: Selection,
    /// What the Space looks like, shown in the expanded bar. Without one the thumbnail is a
    /// quiet blank card.
    pub thumbnail: Option<ImageSource>,
    /// The desktop's own picture (typically the output's wallpaper), drawn under the thumbnail,
    /// cover-fit and clipped to the card's corners. Without a thumbnail it shows alone.
    pub backdrop: Option<ImageSource>,
    /// Its part in a drag: `Target` while the dragged thing is over it, `Accepts` while it could
    /// take the drop but the pointer is elsewhere.
    pub drop: DropState,
}

impl SpaceTile {
    /// A Space that is not the one on show, with no thumbnail and no drag over it.
    pub fn new(key: BarSpaceKey, name: impl Into<String>) -> Self {
        SpaceTile {
            key,
            name: name.into(),
            current: Selection::Unselected,
            thumbnail: None,
            backdrop: None,
            drop: DropState::Idle,
        }
    }

    /// The same Space, marked as the one on show or not.
    pub fn current(self, current: Selection) -> Self {
        SpaceTile { current, ..self }
    }

    /// The same Space with the picture of it to show when the bar expands.
    pub fn thumbnail(self, thumbnail: ImageSource) -> Self {
        SpaceTile {
            thumbnail: Some(thumbnail),
            ..self
        }
    }

    /// The same Space with `backdrop` under its thumbnail, as the Mac shows each desktop's
    /// wallpaper with its windows on top.
    pub fn backdrop(self, backdrop: ImageSource) -> Self {
        SpaceTile {
            backdrop: Some(backdrop),
            ..self
        }
    }

    /// The same Space with `drop` as its part in a drag.
    pub fn in_drag(self, drop: DropState) -> Self {
        SpaceTile { drop, ..self }
    }
}

/// How much of the bar shows: the names alone, or thumbnails with the names under them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum BarExpansion {
    /// A row of names, the current one marked.
    #[default]
    Collapsed,
    /// A thumbnail for each Space, its name under it.
    Expanded,
}

impl BarExpansion {
    /// What the pointer asks for: expanded while it is over the bar, collapsed once it leaves.
    pub(crate) fn under_pointer(over: PointerOver) -> Self {
        match over {
            PointerOver::Bar => BarExpansion::Expanded,
            PointerOver::Elsewhere => BarExpansion::Collapsed,
        }
    }
}

/// Whether the pointer is over the bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum PointerOver {
    /// On the bar or anything in it.
    Bar,
    /// Anywhere else.
    Elsewhere,
}

#[cfg(test)]
mod tests {
    use super::{BarExpansion, BarSpaceKey, PointerOver, SpaceTile};
    use crate::components::content::image_source::ImageSource;
    use ds_core::vocab::{DropState, Selection};

    #[test]
    fn the_bar_is_expanded_exactly_while_the_pointer_is_over_it() {
        const CASES: &[(PointerOver, BarExpansion)] = &[
            (PointerOver::Bar, BarExpansion::Expanded),
            (PointerOver::Elsewhere, BarExpansion::Collapsed),
        ];
        for &(over, want) in CASES {
            assert_eq!(BarExpansion::under_pointer(over), want, "{over:?}");
        }
        assert_eq!(BarExpansion::default(), BarExpansion::Collapsed);
    }

    #[test]
    fn a_new_tile_is_not_current_and_has_nothing_dragged_over_it() {
        let tile = SpaceTile::new(BarSpaceKey("a".to_owned()), "Desktop 1");
        assert_eq!(tile.current, Selection::Unselected);
        assert_eq!(tile.drop, DropState::Idle);
        assert_eq!(tile.thumbnail, None);
        assert_eq!(tile.backdrop, None);
        let marked = tile.current(Selection::Selected).in_drag(DropState::Target);
        assert_eq!(marked.current, Selection::Selected);
        assert_eq!(marked.drop, DropState::Target);
    }

    #[test]
    fn a_backdrop_is_kept_beside_the_thumbnail() {
        let wall = ImageSource("file:///wall.png".to_owned());
        let tile = SpaceTile::new(BarSpaceKey("a".to_owned()), "Desktop 1").backdrop(wall.clone());
        assert_eq!(tile.backdrop, Some(wall));
        assert_eq!(tile.thumbnail, None);
    }
}
