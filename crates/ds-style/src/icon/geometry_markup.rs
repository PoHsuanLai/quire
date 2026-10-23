//! Lucide geometry for the markup and document-view glyphs (a pan hand, crop, a select pointer, a
//! highlighter, a text box, an oval, a line, an arrow, a freehand pen, an outline and a table),
//! transcribed from `lucide-static` 1.47.0 (<https://unpkg.com/lucide-static@1.47.0/icons/>), ISC
//! licence; see `super`. Each child is as published, stroke by attribute like every other glyph.

use super::Icon;
use super::shape::Shape;

/// The children of a glyph this file holds; empty for any other.
pub(super) fn shapes(icon: Icon) -> &'static [Shape] {
    match icon {
        Icon::Hand => HAND,
        Icon::Crop => CROP,
        Icon::MousePointer => MOUSE_POINTER,
        Icon::Highlighter => HIGHLIGHTER,
        Icon::Type => TYPE,
        Icon::Circle => CIRCLE,
        Icon::Line => LINE,
        Icon::ArrowUpRight => ARROW_UP_RIGHT,
        Icon::PenLine => PEN_LINE,
        Icon::ListTree => LIST_TREE,
        Icon::Table => TABLE,
        _ => &[],
    }
}

/// Lucide `hand`: the pan tool.
const HAND: &[Shape] = &[
    Shape::Path("M18 11V6a2 2 0 0 0-2-2a2 2 0 0 0-2 2"),
    Shape::Path("M14 10V4a2 2 0 0 0-2-2a2 2 0 0 0-2 2v2"),
    Shape::Path("M10 10.5V6a2 2 0 0 0-2-2a2 2 0 0 0-2 2v8"),
    Shape::Path(
        "M18 8a2 2 0 1 1 4 0v6a8 8 0 0 1-8 8h-2c-2.8 0-4.5-.86-5.99-2.34l-3.6-3.6a2 2 0 0 1 2.83-2.82L7 15",
    ),
];

/// Lucide `crop`.
const CROP: &[Shape] = &[
    Shape::Path("M6 2v14a2 2 0 0 0 2 2h14"),
    Shape::Path("M18 22V8a2 2 0 0 0-2-2H2"),
];

/// Lucide `mouse-pointer`: the select tool.
const MOUSE_POINTER: &[Shape] = &[
    Shape::Path("M12.586 12.586 19 19"),
    Shape::Path(
        "M3.688 3.037a.497.497 0 0 0-.651.651l6.5 15.999a.501.501 0 0 0 .947-.062l1.569-6.083a2 2 0 0 1 1.448-1.479l6.124-1.579a.5.5 0 0 0 .063-.947z",
    ),
];

/// Lucide `highlighter`.
const HIGHLIGHTER: &[Shape] = &[
    Shape::Path("m9 11-6 6v3h9l3-3"),
    Shape::Path("m22 12-4.6 4.6a2 2 0 0 1-2.8 0l-5.2-5.2a2 2 0 0 1 0-2.8L14 4"),
];

/// Lucide `type`: the text box.
const TYPE: &[Shape] = &[
    Shape::Path("M12 4v16"),
    Shape::Path("M4 7V5a1 1 0 0 1 1-1h14a1 1 0 0 1 1 1v2"),
    Shape::Path("M9 20h6"),
];

/// Lucide `circle`: the oval shape.
const CIRCLE: &[Shape] = &[Shape::Circle {
    cx: "12",
    cy: "12",
    r: "10",
}];

/// Lucide `slash`: a diagonal stroke (Lucide has no glyph named `line`).
const LINE: &[Shape] = &[Shape::Path("M22 2 2 22")];

/// Lucide `arrow-up-right`: the arrow shape.
const ARROW_UP_RIGHT: &[Shape] = &[Shape::Path("M7 7h10v10"), Shape::Path("M7 17 17 7")];

/// Lucide `pen-line`: freehand.
const PEN_LINE: &[Shape] = &[
    Shape::Path("M13 21h8"),
    Shape::Path(
        "M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z",
    ),
];

/// Lucide `list-tree`: an outline, a table of contents.
const LIST_TREE: &[Shape] = &[
    Shape::Path("M8 5h13"),
    Shape::Path("M13 12h8"),
    Shape::Path("M13 19h8"),
    Shape::Path("M3 10a2 2 0 0 0 2 2h3"),
    Shape::Path("M3 5v12a2 2 0 0 0 2 2h3"),
];

/// Lucide `table`: a spreadsheet.
const TABLE: &[Shape] = &[
    Shape::Path("M12 3v18"),
    Shape::Rect {
        x: "3",
        y: "3",
        width: "18",
        height: "18",
        rx: "2",
    },
    Shape::Path("M3 9h18"),
    Shape::Path("M3 15h18"),
];
