//! SpacesBar: Mission Control's strip of Spaces along the top edge (design/04-COMPONENTS.md
//! section 62).

pub mod layout;
pub mod model;
pub mod view;

pub use layout::{BarLayout, SpaceRects};
pub use model::{BarExpansion, BarSpaceKey, SpaceTile};
pub use view::SpacesBar;
