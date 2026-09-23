//! Where a floating surface goes: flip when it does not fit, clamp 8 px inside the bounds
//! (design/01-LAYOUT.md section 8.2). Pure; nothing here measures layout.

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
///
/// `Align::Start` lines the leading edges up exactly; an offset such as a floating menu's
/// `left - 8` is the caller's, applied by moving the anchor. The origin is relative to the
/// bounds' top-left corner. When the wanted side overflows, the opposite side is taken if it
/// fits, or if it has more room than the wanted one; the clamp then keeps the surface 8 px
/// inside the bounds, its top-left edge winning when the surface is larger than the bounds.
pub fn place(anchor: Rect, content: Size, bounds: Rect, want: Placement, gap: Px) -> Placed {
    let side = match want.flip {
        Flip::Never => want.side,
        Flip::Allowed => choose_side(anchor, content, bounds, want.side, gap),
    };
    let at = origin_on(side, want.align, anchor, content, gap);
    let x = clamp_axis(at.x, content.width, bounds.left(), bounds.right());
    let y = clamp_axis(at.y, content.height, bounds.top(), bounds.bottom());
    Placed {
        origin: Point {
            x: x - bounds.left(),
            y: y - bounds.top(),
        },
        side,
    }
}

/// How far every floating surface stays inside its bounds.
const MARGIN: Px = Px(8.0);

/// The side the surface ends on: the wanted one unless it overflows and the opposite does
/// better.
fn choose_side(anchor: Rect, content: Size, bounds: Rect, want: Side, gap: Px) -> Side {
    let wanted = overflow(want, anchor, content, bounds, gap);
    if wanted <= 0.0 {
        return want;
    }
    let other = opposite(want);
    if overflow(other, anchor, content, bounds, gap) < wanted {
        other
    } else {
        want
    }
}

/// How many pixels the surface would cross the 8 px margin by on `side`'s main axis
/// (zero or less when it fits).
fn overflow(side: Side, anchor: Rect, content: Size, bounds: Rect, gap: Px) -> f32 {
    let at = origin_on(side, Align::Start, anchor, content, gap);
    match side {
        Side::Bottom => (at.y + content.height).0 - (bounds.bottom() - MARGIN).0,
        Side::Top => (bounds.top() + MARGIN).0 - at.y.0,
        Side::Right => (at.x + content.width).0 - (bounds.right() - MARGIN).0,
        Side::Left => (bounds.left() + MARGIN).0 - at.x.0,
    }
}

fn opposite(side: Side) -> Side {
    match side {
        Side::Top => Side::Bottom,
        Side::Bottom => Side::Top,
        Side::Left => Side::Right,
        Side::Right => Side::Left,
    }
}

/// The unclamped top-left corner on `side`, lined up by `align`.
fn origin_on(side: Side, align: Align, anchor: Rect, content: Size, gap: Px) -> Point {
    match side {
        Side::Bottom => Point {
            x: along(align, anchor.left(), anchor.size.width, content.width),
            y: anchor.bottom() + gap,
        },
        Side::Top => Point {
            x: along(align, anchor.left(), anchor.size.width, content.width),
            y: anchor.top() - gap - content.height,
        },
        Side::Right => Point {
            x: anchor.right() + gap,
            y: along(align, anchor.top(), anchor.size.height, content.height),
        },
        Side::Left => Point {
            x: anchor.left() - gap - content.width,
            y: along(align, anchor.top(), anchor.size.height, content.height),
        },
    }
}

/// The cross-axis start of a surface of `extent` against an anchor edge `start` of `span`.
fn along(align: Align, start: Px, span: Px, extent: Px) -> Px {
    match align {
        Align::Start => start,
        Align::Center => Px(start.0 + (span.0 - extent.0) / 2.0),
        Align::End => start + span - extent,
    }
}

/// `at` clamped to `[lo + 8, hi - extent - 8]`; the low edge wins when they cross.
fn clamp_axis(at: Px, extent: Px, lo: Px, hi: Px) -> Px {
    let high = (hi - MARGIN - extent).0;
    let low = (lo + MARGIN).0;
    Px(at.0.min(high).max(low))
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
