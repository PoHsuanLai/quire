//! Solid forms of the markup and document-view glyphs (Hand, Crop, MousePointer, Highlighter, Type,
//! Circle, Line, ArrowUpRight, PenLine, ListTree, Table): filled paths on the 24 grid, no stroke
//! (design/08-ICONS.md section 1.2).
//!
//! Each constant says where its geometry comes from. Tabler and Phosphor are MIT, notices in
//! `assets/icons/LICENSE-tabler.txt` and `LICENSE-phosphor.txt`; a Phosphor path is scaled by
//! 3/32 from its 256 grid and rounded to three decimals.

use super::Icon;
use super::shape::Shape;

/// The solid children of a glyph this file holds; empty for any other.
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

/// Phosphor Icons 2.1.1 Fill `hand-fill` (MIT), scaled from its 256 grid to 24.
const HAND: &[Shape] = &[Shape::Solid(
    "M20.25 6 v8.525 c0 4.331 -3.455 7.927 -7.781 7.974 A7.848 7.848 0 0 1 6.806 20.194 C4.762 18.031 2.452 12.75 2.452 12.75 a1.5 1.5 0 0 1 0.612 -2.084 c0.718 -0.375 1.603 -0.079 2.006 0.621 l1.969 3.416 a0.571 0.571 0 0 0 0.562 0.29 l0.011 0 A0.768 0.768 0 0 0 8.25 14.226 V4.5 a1.5 1.5 0 0 1 1.572 -1.5 c0.807 0.038 1.428 0.733 1.428 1.54 V10.5 a0.75 0.75 0 0 0 0.8 0.75 0.766 0.766 0 0 0 0.7 -0.773 V3 a1.5 1.5 0 0 1 1.572 -1.5 c0.807 0.038 1.428 0.733 1.428 1.54 V11.25 a0.75 0.75 0 0 0 0.8 0.75 0.766 0.766 0 0 0 0.7 -0.773 V6.042 c0 -0.807 0.621 -1.5 1.428 -1.54 A1.5 1.5 0 0 1 20.25 6 Z",
)];

/// Phosphor Icons 2.1.1 Fill `crop-fill` (MIT), scaled from its 256 grid to 24.
const CROP: &[Shape] = &[Shape::Solid(
    "M19.5 3 H4.5 A1.5 1.5 0 0 0 3 4.5 V19.5 a1.5 1.5 0 0 0 1.5 1.5 H19.5 a1.5 1.5 0 0 0 1.5 -1.5 V4.5 A1.5 1.5 0 0 0 19.5 3 Z M11.25 7.5 h4.5 a0.75 0.75 0 0 1 0.75 0.75 v4.5 a0.75 0.75 0 0 1 -1.5 0 V9 H11.25 a0.75 0.75 0 0 1 0 -1.5 Z m6.75 9 H16.5 v1.5 a0.75 0.75 0 0 1 -1.5 0 V16.5 H8.25 a0.75 0.75 0 0 1 -0.75 -0.75 V9 H6 a0.75 0.75 0 0 1 0 -1.5 H7.5 V6 a0.75 0.75 0 0 1 1.5 0 v9 h9 a0.75 0.75 0 0 1 0 1.5 Z",
)];

/// Tabler Icons 3.48.0 `filled/pointer` (MIT), verbatim except for the invisible bounding path.
const MOUSE_POINTER: &[Shape] = &[Shape::Solid(
    "M3.039 4.277l3.904 13.563c.185 .837 .92 1.516 1.831 1.642l.17 .016a2.2 2.2 0 0 0 1.982 -1.006l.045 -.078l1.4 -2.072l4.05 4.05a2.067 2.067 0 0 0 2.924 0l1.047 -1.047c.388 -.388 .606 -.913 .606 -1.461l-.008 -.182a2.067 2.067 0 0 0 -.598 -1.28l-4.047 -4.048l2.103 -1.412c.726 -.385 1.18 -1.278 1.053 -2.189a2.2 2.2 0 0 0 -1.701 -1.845l-13.524 -3.89a1 1 0 0 0 -1.236 1.24z",
)];

/// Phosphor Icons 2.1.1 Fill `highlighter-fill` (MIT), scaled from its 256 grid to 24.
const HIGHLIGHTER: &[Shape] = &[Shape::Solid(
    "M23.781 9.969 a0.75 0.75 0 0 0 -1.061 0 L18 14.69 10.06 6.75 l4.72 -4.719 a0.75 0.75 0 1 0 -1.061 -1.061 L9 5.69 A1.5 1.5 0 0 0 8.736 7.453 L6.75 9.44 a1.5 1.5 0 0 0 0 2.121 L7.19 12 1.719 17.469 a0.75 0.75 0 0 0 0.293 1.242 l6.75 2.25 A0.739 0.739 0 0 0 9 21 a0.75 0.75 0 0 0 0.531 -0.219 L12.75 17.56 l0.44 0.44 a1.5 1.5 0 0 0 2.121 0 l1.986 -1.986 A1.5 1.5 0 0 0 19.06 15.75 l4.72 -4.719 A0.75 0.75 0 0 0 23.781 9.969 Z M14.25 16.94 7.81 10.5 9.75 8.56 16.19 15 Z",
)];

/// Phosphor Icons 2.1.1 Fill `text-t-fill` (MIT), scaled from its 256 grid to 24.
const TYPE: &[Shape] = &[Shape::Solid(
    "M19.5 3 H4.5 A1.5 1.5 0 0 0 3 4.5 V19.5 a1.5 1.5 0 0 0 1.5 1.5 H19.5 a1.5 1.5 0 0 0 1.5 -1.5 V4.5 A1.5 1.5 0 0 0 19.5 3 Z M17.25 9 a0.75 0.75 0 0 1 -1.5 0 V8.25 H12.75 v8.25 h1.125 a0.75 0.75 0 0 1 0 1.5 H10.125 a0.75 0.75 0 0 1 0 -1.5 h1.125 V8.25 H8.25 v0.75 a0.75 0.75 0 0 1 -1.5 0 V7.5 a0.75 0.75 0 0 1 0.75 -0.75 h9 a0.75 0.75 0 0 1 0.75 0.75 Z",
)];

/// Tabler Icons 3.48.0 `filled/circle` (MIT), verbatim except for the invisible bounding path.
const CIRCLE: &[Shape] = &[Shape::Solid(
    "M7 3.34a10 10 0 1 1 -4.995 8.984l-.005 -.324l.005 -.324a10 10 0 0 1 4.995 -8.336z",
)];

/// Drawn here: a diagonal capsule 2 wide from (22, 2) to (2, 22), the stroke of Lucide `slash`
/// as a fill.
const LINE: &[Shape] = &[Shape::Solid(
    "M22.707 2.707 2.707 22.707A1 1 0 0 1 1.293 21.293L21.293 1.293A1 1 0 0 1 22.707 2.707z",
)];

/// Phosphor Icons 2.1.1 Fill `arrow-up-right-fill` (MIT), scaled from its 256 grid to 24.
const ARROW_UP_RIGHT: &[Shape] = &[Shape::Solid(
    "M18.75 6 V15.75 a0.75 0.75 0 0 1 -1.281 0.531 L13.125 11.935 6.531 18.531 a0.75 0.75 0 0 1 -1.061 -1.061 L12.065 10.875 7.719 6.531 A0.75 0.75 0 0 1 8.25 5.25 H18 A0.75 0.75 0 0 1 18.75 6 Z",
)];

/// Phosphor Icons 2.1.1 Fill `pencil-simple-line-fill` (MIT), scaled from its 256 grid to 24.
const PEN_LINE: &[Shape] = &[Shape::Solid(
    "M21.311 6.878 17.122 2.69 a1.5 1.5 0 0 0 -2.122 0 L3.44 14.25 A1.487 1.487 0 0 0 3 15.31 V19.5 a1.5 1.5 0 0 0 1.5 1.5 H20.25 a0.75 0.75 0 0 0 0 -1.5 H10.811 l10.5 -10.5 A1.5 1.5 0 0 0 21.311 6.878 Z M18 10.19 13.811 6 l2.25 -2.25 L20.25 7.94 Z",
)];

/// Tabler Icons 3.48.0 `filled/list` (MIT), verbatim except for the invisible bounding path.
const LIST_TREE: &[Shape] = &[
    Shape::Solid("M21 6a1 1 0 0 1 -1 1h-10a1 1 0 1 1 0 -2h10a1 1 0 0 1 1 1"),
    Shape::Solid("M21 12a1 1 0 0 1 -1 1h-10a1 1 0 0 1 0 -2h10a1 1 0 0 1 1 1"),
    Shape::Solid("M21 18a1 1 0 0 1 -1 1h-10a1 1 0 0 1 0 -2h10a1 1 0 0 1 1 1"),
    Shape::Solid(
        "M7 5.995v.02c0 1.099 -.895 1.99 -2 1.99s-2 -.891 -2 -1.99v-.02c0 -1.099 .895 -1.99 2 -1.99s2 .891 2 1.99",
    ),
    Shape::Solid(
        "M7 11.995v.02c0 1.099 -.895 1.99 -2 1.99s-2 -.891 -2 -1.99v-.02c0 -1.099 .895 -1.99 2 -1.99s2 .891 2 1.99",
    ),
    Shape::Solid(
        "M7 17.995v.02c0 1.099 -.895 1.99 -2 1.99s-2 -.891 -2 -1.99v-.02c0 -1.099 .895 -1.99 2 -1.99s2 .891 2 1.99",
    ),
];

/// Tabler Icons 3.48.0 `filled/table` (MIT), verbatim except for the invisible bounding path.
const TABLE: &[Shape] = &[
    Shape::Solid(
        "M4 11h4a1 1 0 0 1 1 1v8a1 1 0 0 1 -1 1h-2a3 3 0 0 1 -2.995 -2.824l-.005 -.176v-6a1 1 0 0 1 1 -1z",
    ),
    Shape::Solid(
        "M21 12v6a3 3 0 0 1 -2.824 2.995l-.176 .005h-6a1 1 0 0 1 -1 -1v-8a1 1 0 0 1 1 -1h8a1 1 0 0 1 1 1z",
    ),
    Shape::Solid(
        "M18 3a3 3 0 0 1 2.995 2.824l.005 .176v2a1 1 0 0 1 -1 1h-8a1 1 0 0 1 -1 -1v-4a1 1 0 0 1 1 -1h6z",
    ),
    Shape::Solid(
        "M9 4v4a1 1 0 0 1 -1 1h-4a1 1 0 0 1 -1 -1v-2a3 3 0 0 1 2.824 -2.995l.176 -.005h2a1 1 0 0 1 1 1z",
    ),
];
