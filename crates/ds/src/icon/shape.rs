//! One child of a glyph's `svg`, as data.

/// One SVG child. Numbers are the design's decimal text, so `.6` stays `.6`.
///
/// A rect attribute the design omits is `"0"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Shape {
    /// An outline, the path `d`.
    Path(&'static str),
    /// A path filled in the ink as well as stroked, so its silhouette is the outline's: a
    /// symbol's filled variant (the moon of a Focus that is on, design/26 G23).
    Solid(&'static str),
    /// A circle on the 24 grid.
    Circle {
        /// Centre x.
        cx: &'static str,
        /// Centre y.
        cy: &'static str,
        /// Radius.
        r: &'static str,
    },
    /// A rectangle on the 24 grid.
    Rect {
        /// Left.
        x: &'static str,
        /// Top.
        y: &'static str,
        /// Width.
        width: &'static str,
        /// Height.
        height: &'static str,
        /// Corner radius; `"0"` when the design omits it.
        rx: &'static str,
    },
}
