//! Lucide geometry for glyphs a consumer's actions need beyond the mailo and shell sets
//! (FINDINGS "mailo gaps"), transcribed from `lucide-static` 1.47.0
//! (<https://unpkg.com/lucide-static@1.47.0/icons/>), ISC licence; see `super`. Each child is as
//! published, stroke by attribute like every other glyph.

use super::shape::Shape;

/// Lucide `printer`.
pub(super) const PRINTER: &[Shape] = &[
    Shape::Path("M6 18H4a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2"),
    Shape::Path("M6 9V3a1 1 0 0 1 1-1h10a1 1 0 0 1 1 1v6"),
    Shape::Rect {
        x: "6",
        y: "14",
        width: "12",
        height: "8",
        rx: "1",
    },
];

/// Lucide `folder-input`.
pub(super) const FOLDER_INPUT: &[Shape] = &[
    Shape::Path(
        "M2 9V5a2 2 0 0 1 2-2h3.9a2 2 0 0 1 1.69.9l.81 1.2a2 2 0 0 0 1.67.9H20a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2v-1",
    ),
    Shape::Path("M2 13h10"),
    Shape::Path("m9 16 3-3-3-3"),
];
