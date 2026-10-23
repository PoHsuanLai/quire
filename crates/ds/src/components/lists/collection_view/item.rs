//! What a caller hands a `CollectionView` for each tile.

use crate::components::content::file_icon::FileIconChoice;
use crate::components::content::icon_source::IconSource;
use crate::components::content::image_source::ImageSource;
use dioxus::prelude::Element;
use ds_core::vocab::DropState;

/// The picture on a tile.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Thumbnail {
    /// A picture of the item itself (a photo, a page of a document), fitted inside the
    /// thumbnail's square.
    Image(ImageSource),
    /// A glyph or an icon standing for the kind of item (a folder), drawn at the thumbnail's
    /// size.
    Icon(IconSource),
    /// A file's icon (`ds::FileIcon`): a standard page, a folder, a picture on a page frame or
    /// finished artwork, drawn whole at the thumbnail's size like an icon and never cropped to a
    /// frame of the tile's.
    FileIcon(FileIconChoice),
}

/// One tile: what identifies it, its picture and its name.
#[derive(Debug, Clone, PartialEq)]
pub struct CollectionItem<K> {
    /// The tile's identity, kept across renders: the roster plays a tile's entrance and exit
    /// by it.
    pub key: K,
    /// The name under the thumbnail, clipped from the middle to the tile's width so a file
    /// name keeps its start and its extension. It is the tile's accessible name and what
    /// type-to-select matches.
    pub caption: String,
    /// The picture above it.
    pub thumbnail: Thumbnail,
    /// The same place held by a field the person types in (a file renamed where it stands: a
    /// `TextField` with `FieldBezel::Plain`), as `Row`'s `edit`. The tile takes no press
    /// meanwhile (`data-editing`), and the keys, presses and drags inside the field stay the
    /// field's.
    pub edit: Option<Element>,
    /// What sits centred under the caption (Finder's tag dots). It takes the room the tile
    /// already keeps under the caption, so a tile with and a tile without one have the same
    /// pitch.
    pub badge: Option<Element>,
    /// The tile's part in a drag, as `Row`'s `RowState::drop`: `Accepts` and `Target` make it
    /// a place that takes a drop (it writes `data-drop` and the view reports the pointer over
    /// it), `Source` dims it as the thing being dragged, `Idle` is neither. The owner sets
    /// `Target` on the tile under the pointer.
    pub drop: DropState,
}

impl<K> CollectionItem<K> {
    /// A tile with no field in it.
    pub fn new(key: K, caption: impl Into<String>, thumbnail: Thumbnail) -> Self {
        CollectionItem {
            key,
            caption: caption.into(),
            thumbnail,
            edit: None,
            badge: None,
            drop: DropState::Idle,
        }
    }

    /// The same tile with `field` in place of its caption.
    pub fn editing(self, field: Element) -> Self {
        CollectionItem {
            edit: Some(field),
            ..self
        }
    }

    /// The same tile with `badge` centred under its caption.
    pub fn badged(self, badge: Element) -> Self {
        CollectionItem {
            badge: Some(badge),
            ..self
        }
    }

    /// The same tile with `drop` as its part in a drag.
    pub fn in_drag(self, drop: DropState) -> Self {
        CollectionItem { drop, ..self }
    }
}
