//! `Kits`: the kits a surface is drawn with, in cascade order.

use super::blocks::token_blocks;
use super::model::{Kit, Sheet};
use crate::css::document::document;
use crate::tokens::{easing::EasingToken, set::TokenSet, timing::DurationToken};
use std::collections::HashSet;

/// The kits in play, ordered by rank.
#[derive(Debug, Clone)]
pub struct Kits {
    kits: Vec<&'static Kit>,
}

/// What the linter accepts, merged from every kit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnownNames {
    /// Every custom property, `--` included.
    pub vars: HashSet<String>,
    /// Every keyframes name, lower-cased.
    pub keyframes: HashSet<String>,
    /// The durations the details grammar plays a moment with.
    pub grammar_durations: Vec<DurationToken>,
    /// The easings the details grammar plays a moment along.
    pub grammar_easings: Vec<EasingToken>,
}

impl Kits {
    /// `kits` in cascade order: fixed by each kit's rank, not by the order given.
    pub fn of(kits: &[&'static Kit]) -> Kits {
        let mut kits = kits.to_vec();
        kits.sort_by_key(|kit| kit.rank);
        Kits { kits }
    }

    /// The whole stylesheet: every kit's sections in cascade order, each under its marker.
    pub fn stylesheet(&self) -> String {
        let sections: Vec<(&str, String)> = self
            .kits
            .iter()
            .flat_map(|kit| kit.sections)
            .map(|section| (section.name, (section.css)(self).into_owned()))
            .collect();
        document(&sections)
    }

    /// The component sheets the kits add to the `components` section, in kit order.
    pub fn sheets(&self) -> Vec<Sheet> {
        self.kits
            .iter()
            .flat_map(|kit| kit.sheets.iter().copied())
            .collect()
    }

    /// The `.ds` token blocks of every kit's families: the base declarations, then what the dark
    /// scheme, the editorial typeface and each motion level change.
    pub fn token_blocks(&self) -> String {
        token_blocks(&self.token_sets())
    }

    /// Every family, placed in the order the `.ds` block writes them.
    pub fn token_sets(&self) -> Vec<TokenSet> {
        let mut sets: Vec<TokenSet> = self
            .kits
            .iter()
            .flat_map(|kit| kit.tokens.iter().copied())
            .collect();
        sets.sort_by_key(TokenSet::place);
        sets
    }

    /// Everything the linter may accept because of these kits.
    pub fn vocabulary(&self) -> KnownNames {
        let tokens = self
            .kits
            .iter()
            .flat_map(|kit| kit.tokens)
            .flat_map(TokenSet::vars)
            .map(|var| var.as_str().to_owned());
        let inline = self
            .kits
            .iter()
            .flat_map(|kit| (kit.vocabulary.inline_vars)());
        let keyframes = self
            .kits
            .iter()
            .flat_map(|kit| (kit.vocabulary.keyframes)())
            .map(|name| name.to_ascii_lowercase());
        KnownNames {
            vars: tokens.chain(inline).collect(),
            keyframes: keyframes.collect(),
            grammar_durations: self
                .kits
                .iter()
                .flat_map(|kit| kit.vocabulary.grammar_durations.iter().copied())
                .collect(),
            grammar_easings: self
                .kits
                .iter()
                .flat_map(|kit| kit.vocabulary.grammar_easings.iter().copied())
                .collect(),
        }
    }
}
