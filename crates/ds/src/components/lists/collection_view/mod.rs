//! CollectionView: Finder's icon view, a grid of tiles (a thumbnail over a caption) of any
//! number that never builds them all.

pub(crate) mod model;
pub(crate) mod tile;
pub(crate) mod view;

mod act;
mod item;
pub(crate) mod landing;
mod size;
mod window;

pub use item::{CollectionItem, Thumbnail};
pub use size::{CollectionExtent, CollectionSize, Flow, ThumbAspect};
pub use view::CollectionView;
