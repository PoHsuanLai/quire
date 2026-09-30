//! The motion layer's kit: the keyframes section and the vocabulary the linter checks a
//! stylesheet's animations against.

use crate::motion::{
    anim::Anim,
    css::{ALIAS, motion_css},
    detail::grammar::{GRAMMAR_DURATIONS, GRAMMAR_EASINGS},
};
use ds_style::kit::{Kit, KitRank, Kits, Section, Vocabulary};
use std::borrow::Cow;

/// The motion layer's contribution to the stylesheet and the linter.
pub static KIT: Kit = Kit {
    rank: KitRank::Motion,
    tokens: &[],
    sections: &[Section {
        name: "motion",
        css: keyframes,
    }],
    vocabulary: Vocabulary {
        keyframes: keyframe_names,
        grammar_durations: &GRAMMAR_DURATIONS,
        grammar_easings: GRAMMAR_EASINGS,
        ..Vocabulary::NONE
    },
};

fn keyframes(_: &Kits) -> Cow<'static, str> {
    Cow::Owned(motion_css())
}

/// Every keyframes name an [`Anim`] plays, and its `X--b` restart alias.
fn keyframe_names() -> Vec<String> {
    Anim::ALL
        .iter()
        .map(|anim| anim.recipe().keyframes)
        .flat_map(|name| [name.to_owned(), format!("{name}{ALIAS}")])
        .collect()
}
