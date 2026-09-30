//! The components' kit (the utilities, then every component sheet in its one registered order),
//! and the kits a surface is drawn with.

use crate::assembly::sheets::SHEETS;
use ds_style::css::document::{placed, sheets, utilities_css};
use ds_style::kit::{Kit, KitRank, Kits, STYLE_KIT, Section, Vocabulary};
use std::borrow::Cow;

/// The components' contribution to the stylesheet.
pub(crate) static KIT: Kit = Kit {
    rank: KitRank::Components,
    tokens: &[],
    sections: &[
        Section {
            name: "utilities",
            css: utilities,
        },
        Section {
            name: "components",
            css: components,
        },
    ],
    sheets: &[],
    vocabulary: Vocabulary::NONE,
};

/// Every kit the design system ships: style, motion, the components and the shell.
pub fn kits() -> Kits {
    Kits::of(&[
        &STYLE_KIT,
        &ds_motion::kit::KIT,
        &KIT,
        &crate::shell::kit::KIT,
    ])
}

fn utilities(_: &Kits) -> Cow<'static, str> {
    Cow::Owned(utilities_css())
}

fn components(kits: &Kits) -> Cow<'static, str> {
    Cow::Owned(sheets(&placed(&SHEETS, &kits.sheets())))
}
