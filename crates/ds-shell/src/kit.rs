//! The kit of the app-facing parts: their component sheets, placed in the components section, each
//! after the sheet it follows. The parts add no tokens.

use crate::sheets::SHEETS;
use ds_style::css::document::placed;
use ds_style::kit::{Kit, KitRank, Kits, Vocabulary};
use std::sync::LazyLock;

/// These parts' contribution to the stylesheet and the linter.
pub static KIT: Kit = Kit {
    rank: KitRank::Shell,
    tokens: &[],
    sections: &[],
    sheets: &SHEETS,
    vocabulary: Vocabulary::NONE,
};

/// Every kit these parts are drawn with: the design system's (style, motion, the components) and
/// this crate's. A crate with more kits of its own lists these and adds them.
pub fn kits() -> Kits {
    Kits::of(&[
        &ds_style::kit::STYLE_KIT,
        &ds_motion::kit::KIT,
        &ds::KIT,
        &KIT,
    ])
}

/// Every rule these parts draw with, in cascade order. Built once, on first use.
pub fn stylesheet() -> &'static str {
    static SHEET: LazyLock<String> = LazyLock::new(|| kits().stylesheet());
    SHEET.as_str()
}

/// Every component sheet, the design system's and these parts', `(name, css)`, in the order the
/// stylesheet appends them.
pub fn component_sheets() -> Vec<(&'static str, &'static str)> {
    placed(&ds::component_sheets(), &SHEETS)
}
