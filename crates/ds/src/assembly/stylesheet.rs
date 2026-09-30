//! The whole stylesheet, generated once from the kits in their cascade order: reset, tokens (with
//! the level blocks), accents, materials, the shapes (squircle, plates, floor), the frame ground,
//! keyframes with their aliases and pulse classes, utilities with the grain tile, then every
//! component sheet in its fixed order (`assembly::sheets`).

use crate::assembly::kit::kits;
use crate::assembly::sheets::SHEETS;
use ds_style::css::document::placed;
use std::sync::LazyLock;

/// Every rule the design system draws with, in cascade order. Built once, on first use.
pub fn stylesheet() -> &'static str {
    static SHEET: LazyLock<String> = LazyLock::new(|| kits().stylesheet());
    SHEET.as_str()
}

/// Every component sheet, `(name, css)`, in the order the stylesheet appends them.
pub fn component_sheets() -> Vec<(&'static str, &'static str)> {
    placed(&SHEETS, &kits().sheets())
}
