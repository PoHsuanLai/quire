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

// mailo gaps 6: the "more" glyphs, for a row's or a header's overflow menu (mailo's folder rows
// drew a literal `⋯`, which takes the font's weight and baseline rather than the glyph grid's).

/// Lucide `ellipsis`: three dots across.
pub(super) const ELLIPSIS: &[Shape] = &[
    Shape::Circle {
        cx: "12",
        cy: "12",
        r: "1",
    },
    Shape::Circle {
        cx: "19",
        cy: "12",
        r: "1",
    },
    Shape::Circle {
        cx: "5",
        cy: "12",
        r: "1",
    },
];

/// Lucide `ellipsis-vertical`: three dots down.
pub(super) const ELLIPSIS_VERTICAL: &[Shape] = &[
    Shape::Circle {
        cx: "12",
        cy: "12",
        r: "1",
    },
    Shape::Circle {
        cx: "12",
        cy: "5",
        r: "1",
    },
    Shape::Circle {
        cx: "12",
        cy: "19",
        r: "1",
    },
];

// Lock and switcher parts (M11): the lock field's enter button and its caps-lock mark.

/// Lucide `arrow-right`.
pub(super) const ARROW_RIGHT: &[Shape] = &[Shape::Path("M5 12h14"), Shape::Path("m12 5 7 7-7 7")];

/// Lucide `arrow-big-up-dash` in its square-cornered form: the caps-lock key's arrow over its
/// bar.
pub(super) const CAPS_LOCK: &[Shape] = &[
    Shape::Path("M9 19h6"),
    Shape::Path("M9 15v-3H5l7-7 7 7h-4v3H9z"),
];

// Launcher v2 (sill Q296): the clipboard, emoji and web providers' glyphs.

/// Lucide `clipboard` (its clip's `ry` equals its `rx`, so the rect is as published).
pub(super) const CLIPBOARD: &[Shape] = &[
    Shape::Rect {
        x: "8",
        y: "2",
        width: "8",
        height: "4",
        rx: "1",
    },
    Shape::Path("M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"),
];

/// Lucide `smile`.
pub(super) const SMILE: &[Shape] = &[
    Shape::Path("M15 10V9"),
    Shape::Path("M16.472 15a6 6 0 01-8.943 0"),
    Shape::Path("M9 10V9"),
    Shape::Circle {
        cx: "12",
        cy: "12",
        r: "10",
    },
];

/// Lucide `globe`.
pub(super) const GLOBE: &[Shape] = &[
    Shape::Circle {
        cx: "12",
        cy: "12",
        r: "10",
    },
    Shape::Path("M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20"),
    Shape::Path("M2 12h20"),
];
