//! A rectangle in the document's logical pixels, as edges.

/// A rectangle as its four edges.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    /// The left edge.
    pub left: f32,
    /// The top edge.
    pub top: f32,
    /// The right edge.
    pub right: f32,
    /// The bottom edge.
    pub bottom: f32,
}

impl Bounds {
    /// The rectangle at `x`, `y` of `width` by `height`.
    pub fn at(x: f32, y: f32, width: f32, height: f32) -> Self {
        Bounds {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        }
    }

    /// The width.
    pub fn width(&self) -> f32 {
        self.right - self.left
    }

    /// The height.
    pub fn height(&self) -> f32 {
        self.bottom - self.top
    }

    /// The vertical middle.
    pub fn middle(&self) -> f32 {
        (self.top + self.bottom) / 2.0
    }

    /// Whether the vertical middle of `other` lies between this rectangle's top and bottom: it
    /// is on a line this box spans.
    pub fn spans_middle_of(&self, other: &Bounds) -> bool {
        (self.top..=self.bottom).contains(&other.middle())
    }

    /// The smallest rectangle holding both.
    pub fn union(&self, other: &Bounds) -> Bounds {
        Bounds {
            left: self.left.min(other.left),
            top: self.top.min(other.top),
            right: self.right.max(other.right),
            bottom: self.bottom.max(other.bottom),
        }
    }
}
