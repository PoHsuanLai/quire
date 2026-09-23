//! Logical-pixel geometry and floating-surface placement (design/01-LAYOUT.md section 8,
//! design/06-INTERACTIONS.md section 4).

pub mod measure;
pub mod placement;
pub mod units;

pub use measure::{Anchor, MountedRef, RectProbe, use_rect};
pub use placement::{Align, Flip, Placed, Placement, PopoverRequest, Side, place};
pub use units::{Point, Px, Rect, Size};
