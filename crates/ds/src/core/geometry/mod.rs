//! Logical-pixel geometry and floating-surface placement (design/01-LAYOUT.md section 8,
//! design/06-INTERACTIONS.md section 4).

pub mod placement;

pub mod scale;
pub mod units;

pub use crate::host::measure::{Anchor, HostMeasure, Measured, MountedRef, RectProbe, use_rect};
pub use crate::host::reveal::{HostReveal, ScrollSpan, Scrolled, nearest_scroll};
pub use placement::{Align, Flip, Placed, Placement, PopoverRequest, Side, place};
pub use scale::{Grid, Scale};
pub use units::{Point, Px, Rect, Size};
