//! Lucide geometry for the text-formatting glyphs a format bar draws, transcribed from
//! `lucide-static` 1.47.0 (<https://unpkg.com/lucide-static@1.47.0/icons/>), ISC licence; see
//! `super`. A `line` is written as the path it draws, since a glyph's children are paths, circles
//! and rects; the stroke is the one weight every glyph has.

use super::shape::Shape;

/// Lucide `bold`.
pub(super) const BOLD: &[Shape] = &[Shape::Path(
    "M6 12h9a4 4 0 0 1 0 8H7a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1h7a4 4 0 0 1 0 8",
)];

/// Lucide `italic`.
pub(super) const ITALIC: &[Shape] = &[
    Shape::Path("M19 4H10"),
    Shape::Path("M14 20H5"),
    Shape::Path("M15 4 9 20"),
];

/// Lucide `underline`.
pub(super) const UNDERLINE: &[Shape] = &[
    Shape::Path("M6 4v6a6 6 0 0 0 12 0V4"),
    Shape::Path("M4 20h16"),
];

/// Lucide `strikethrough`.
pub(super) const STRIKE: &[Shape] = &[
    Shape::Path("M16 4H9a3 3 0 0 0-2.83 4"),
    Shape::Path("M14 12a4 4 0 0 1 0 8H6"),
    Shape::Path("M4 12h16"),
];

/// Lucide `code`.
pub(super) const CODE: &[Shape] = &[Shape::Path("m16 18 6-6-6-6"), Shape::Path("m8 6-6 6 6 6")];
