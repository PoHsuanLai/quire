//! `Kit`: one layer's contribution to the stylesheet and the linter.

use super::kits::Kits;
use crate::tokens::{easing::EasingToken, set::TokenSet, timing::DurationToken};
use ds_core::word::Word;
use std::borrow::Cow;

/// Where a kit's sections sit in the cascade, first to last. A user's own style is last, so its
/// rules win by order and never need `!important`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Word)]
pub enum KitRank {
    /// Reset, tokens, accents, materials, shapes and the frame ground.
    Style,
    /// Keyframes and the classes that play them.
    Motion,
    /// The generic components and the utilities.
    Components,
    /// The shell's own parts.
    Shell,
    /// The person's own stylesheet.
    User,
}

/// One named part of the stylesheet. It is handed every kit, so the tokens section can write
/// the declarations of all of them.
#[derive(Debug, Clone, Copy)]
pub struct Section {
    /// The marker the stylesheet writes above it: `/* == name == */`.
    pub name: &'static str,
    /// The section's CSS.
    pub css: fn(&Kits) -> Cow<'static, str>,
}

/// One component's stylesheet, contributed by a crate that owns components the `components`
/// section does not list, and placed in it right after the sheet called `after` (or after another
/// contributed sheet of that name). The place is the cascade order the sheet had when it was one of
/// the components' own, so a crate above `ds` adds sheets without changing what wins.
#[derive(Debug, Clone, Copy)]
pub struct Sheet {
    /// The name written in the marker `/* -- name -- */`.
    pub name: &'static str,
    /// The sheet's CSS.
    pub css: &'static str,
    /// The name of the sheet it follows.
    pub after: &'static str,
}

/// What a kit tells the linter beyond its tokens.
#[derive(Debug, Clone, Copy)]
pub struct Vocabulary {
    /// Every keyframes name the kit's animations play, each alias included.
    pub keyframes: fn() -> Vec<String>,
    /// Custom properties a section declares for itself or a component writes inline, which no
    /// token family lists.
    pub inline_vars: fn() -> Vec<String>,
    /// The durations the details grammar plays a moment with.
    pub grammar_durations: &'static [DurationToken],
    /// The easings the details grammar plays a moment along.
    pub grammar_easings: &'static [EasingToken],
}

impl Vocabulary {
    /// A kit with nothing to add.
    pub const NONE: Vocabulary = Vocabulary {
        keyframes: Vec::new,
        inline_vars: Vec::new,
        grammar_durations: &[],
        grammar_easings: &[],
    };
}

/// One layer's contribution.
#[derive(Debug, Clone, Copy)]
pub struct Kit {
    /// Where its sections sit in the cascade.
    pub rank: KitRank,
    /// The token families it declares.
    pub tokens: &'static [TokenSet],
    /// The stylesheet sections it adds, in order.
    pub sections: &'static [Section],
    /// The component sheets it adds to the `components` section, each after a sheet it names.
    pub sheets: &'static [Sheet],
    /// What the linter may accept because of it.
    pub vocabulary: Vocabulary,
}
