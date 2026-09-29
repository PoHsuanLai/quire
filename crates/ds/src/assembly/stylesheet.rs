//! The whole stylesheet, generated once: every layer's sections in the cascade's order, then
//! every component sheet from the one ordered registration (`assembly::sheets`).

use crate::assembly::sheets::SHEETS;
use crate::css::RESET;
use crate::css::accents_css::accents_css;
use crate::css::document::{document, sheets, utilities_css};
use crate::css::ground_css::ground_css;
use crate::css::materials_css::materials_css;
use crate::css::motion_css::motion_css;
use crate::css::shape_css::shape_css;
use crate::css::tokens_css::tokens_css;
use std::sync::LazyLock;

/// Every rule the design system draws with, in cascade order. Built once, on first use.
pub fn stylesheet() -> &'static str {
    static SHEET: LazyLock<String> = LazyLock::new(build);
    SHEET.as_str()
}

/// Every component sheet, `(name, css)`, in the order the stylesheet appends them.
pub fn component_sheets() -> &'static [(&'static str, &'static str)] {
    &SHEETS
}

/// Reset, tokens (with the level blocks), accents, materials, the shapes (squircle, plates,
/// floor), the frame ground, keyframes with their aliases and pulse classes, utilities with the
/// grain tile, then every component in its fixed order.
fn build() -> String {
    document(&[
        ("reset", RESET.trim_end().to_owned()),
        ("tokens", tokens_css()),
        ("accents", accents_css()),
        ("materials", materials_css()),
        ("shapes", shape_css()),
        ("ground", ground_css()),
        ("motion", motion_css()),
        ("utilities", utilities_css()),
        ("components", sheets(&SHEETS)),
    ])
}
