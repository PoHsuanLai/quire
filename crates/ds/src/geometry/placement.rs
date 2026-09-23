//! Where a floating surface goes: flip when it does not fit, clamp 8 px inside the bounds
//! (design/01-LAYOUT.md section 8.2). Pure; nothing here measures layout.
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

use super::units::{Point, Px, Rect, Size};

/// Which side of the anchor the surface sits on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    /// Above the anchor.
    Top,
    /// Below the anchor.
    Bottom,
    /// Left of the anchor.
    Left,
    /// Right of the anchor.
    Right,
}

/// How the surface lines up with the anchor along that side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Align {
    /// Leading edges aligned.
    Start,
    /// Centred.
    Center,
    /// Trailing edges aligned.
    End,
}

/// Whether the surface may move to the opposite side when it does not fit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Flip {
    /// Flip, then clamp: menus.
    Allowed,
    /// Only clamp: hover cards slide along the edge instead (design/06-INTERACTIONS.md
    /// section 3).
    Never,
}

/// Where a surface wants to be. The prose `Placement::BottomEnd` in the component docs is
/// `Placement::new(Side::Bottom, Align::End)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Placement {
    /// The side of the anchor.
    pub side: Side,
    /// The alignment along it.
    pub align: Align,
    /// Whether it may flip.
    pub flip: Flip,
}

impl Placement {
    /// `side` and `align`, flipping allowed.
    pub const fn new(side: Side, align: Align) -> Self {
        Placement {
            side,
            align,
            flip: Flip::Allowed,
        }
    }

    /// The same placement, never flipping.
    pub const fn no_flip(self) -> Self {
        Placement {
            flip: Flip::Never,
            ..self
        }
    }
}

/// Where the surface ended up.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placed {
    /// Its top-left corner, in the bounds' coordinates.
    pub origin: Point,
    /// The side it ended on, after any flip.
    pub side: Side,
}

/// Place `content` against `anchor` inside `bounds`, `gap` away from it: flip when it does
/// not fit (unless [`Flip::Never`]), then clamp 8 px inside the bounds.
pub fn place(anchor: Rect, content: Size, bounds: Rect, want: Placement, gap: Px) -> Placed {
    todo!()
}

/// What a floating component asks the host for when it must be its own surface: shell-host
/// maps it to an `xdg_positioner` (anchor rect, gravity, flip and slide constraints).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PopoverRequest {
    /// The anchor, in the parent surface's coordinates.
    pub anchor: Rect,
    /// Where the popup wants to be.
    pub placement: Placement,
    /// The popup's size.
    pub size: Size,
}
