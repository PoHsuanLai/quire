//! `TokenSet`: a token family as data, so the stylesheet and the linter can read every family
//! without naming it.

use super::name::VarName;
use super::token::{Token, TokenScope};
use crate::style::appearance::typeface::Typeface;

/// Where a family's declarations sit inside the `.ds` block, in the order the block writes them.
/// A kit places each of its sets, so the block's text does not depend on which crate a family
/// lives in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Place {
    /// The palette, the label hues and the shadows: everything that follows the scheme.
    Scheme,
    /// Corner radii.
    Shape,
    /// The size ladder: each control size's geometry, then the shell's sizes on it.
    Ladder,
    /// Identity swatches, spacing, the sizes that no typeface moves, layers and opacities.
    Scale,
    /// Consumer-tuned geometry: the shell's type scale, dock, OSD, notifications and widgets.
    Metrics,
    /// Device-pixel lines and the icon plate's shares.
    Pixel,
    /// The emoji face's stack.
    Face,
    /// Font families, the voice tokens and the sizes that follow the typeface.
    Typeface,
    /// Durations, delays, easings and scalars.
    Motion,
}

/// Which members of a family a set lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Only {
    /// Every member.
    All,
    /// The members the typeface does not move.
    TypefaceFixed,
    /// The members the typeface moves.
    TypefaceVarying,
}

/// One family's tokens, type-erased.
#[derive(Debug, Clone, Copy)]
pub struct TokenSet {
    place: Place,
    only: Only,
    vars: fn() -> Vec<VarName>,
    declared: fn(TokenScope, Only) -> Vec<(VarName, String)>,
}

impl TokenSet {
    /// Every member of `T`, placed with the scale.
    pub const fn of<T: Token>() -> TokenSet {
        TokenSet {
            place: Place::Scale,
            only: Only::All,
            vars: vars_of::<T>,
            declared: declared_of::<T>,
        }
    }

    /// `self` placed at `place`.
    pub const fn at(self, place: Place) -> TokenSet {
        TokenSet { place, ..self }
    }

    /// `self` listing only `only`'s members in the stylesheet.
    pub const fn only(self, only: Only) -> TokenSet {
        TokenSet { only, ..self }
    }

    /// Where the set's declarations sit in the `.ds` block.
    pub const fn place(&self) -> Place {
        self.place
    }

    /// Every custom property the family declares or reads: each member's, and each tuned
    /// member's input. The linter's vocabulary; a set placed for a subset still names them all.
    pub fn vars(&self) -> Vec<VarName> {
        (self.vars)()
    }

    /// The declarations the set writes in `scope`, `(property, value)`.
    pub fn declarations(&self, scope: TokenScope) -> Vec<(VarName, String)> {
        (self.declared)(scope, self.only)
    }
}

fn vars_of<T: Token>() -> Vec<VarName> {
    T::ALL
        .iter()
        .flat_map(|token| std::iter::once(token.var()).chain(token.input()))
        .collect()
}

fn declared_of<T: Token>(scope: TokenScope, only: Only) -> Vec<(VarName, String)> {
    T::ALL
        .iter()
        .filter(|token| keeps(**token, only, scope))
        .map(|token| (token.var(), token.css_value(scope).to_string()))
        .collect()
}

/// Whether `token` is a member `only` lists: a member follows the typeface when its value
/// differs between the two typefaces.
fn keeps<T: Token>(token: T, only: Only, scope: TokenScope) -> bool {
    let moved = || {
        token.css_value(scope.in_typeface(Typeface::System))
            != token.css_value(scope.in_typeface(Typeface::Editorial))
    };
    match only {
        Only::All => true,
        Only::TypefaceFixed => !moved(),
        Only::TypefaceVarying => moved(),
    }
}
