//! `Token`: one family of design tokens, an enum whose variants are custom properties.
//!
//! `#[derive(Word, Token)]` is the only way to write a family: the enum-level
//! `#[token(prefix = "dur-", kind = fixed | tuned)]` and each variant's values generate `var`,
//! `css_value` and (tuned) `input`, so a token cannot reach CSS without also reaching the
//! linter's vocabulary (`TokenSet::of::<T>()` lists them all).

use super::name::VarName;
use crate::appearance::{motion::MotionLevel, theme::Scheme, typeface::Typeface};
use ds_core::word::Word;
use std::borrow::Cow;

pub use ds_core_derive::Token;

/// Whether the stylesheet alone owns a token's value, or a consumer may write it inline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    /// The stylesheet declares the value.
    Fixed,
    /// The stylesheet declares the token as `var(--input, default)`; a consumer writes
    /// `--input` on any element around its surface, and every nested `.ds` re-reads it.
    Tuned,
}

/// What a token's value can follow: the colour scheme, the motion level, the typeface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TokenScope {
    /// Light or dark.
    pub scheme: Scheme,
    /// The motion level.
    pub motion: MotionLevel,
    /// The voice of the type.
    pub typeface: Typeface,
}

impl TokenScope {
    /// The scope `.ds` itself is declared in: light, Standard motion, the system
    /// typeface.
    pub const BASE: TokenScope = TokenScope {
        scheme: Scheme::Light,
        motion: MotionLevel::Standard,
        typeface: Typeface::System,
    };

    /// `self` in `scheme`.
    pub const fn in_scheme(self, scheme: Scheme) -> TokenScope {
        TokenScope { scheme, ..self }
    }

    /// `self` at `motion`.
    pub const fn at(self, motion: MotionLevel) -> TokenScope {
        TokenScope { motion, ..self }
    }

    /// `self` in `typeface`.
    pub const fn in_typeface(self, typeface: Typeface) -> TokenScope {
        TokenScope { typeface, ..self }
    }
}

/// A token's value as the stylesheet writes it: `90ms`, `#e9ece6`, `var(--input,13px)`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CssValue(Cow<'static, str>);

impl CssValue {
    /// A value that is one literal in the scope.
    pub const fn fixed(text: &'static str) -> CssValue {
        CssValue(Cow::Borrowed(text))
    }

    /// A value computed for the scope.
    pub fn computed(text: String) -> CssValue {
        CssValue(Cow::Owned(text))
    }

    /// A tuned token: its consumer-written `input`, with `default` behind it.
    pub fn tuned(input: VarName, default: &str) -> CssValue {
        CssValue::computed(format!("var({},{default})", input.as_str()))
    }

    /// The text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for CssValue {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.write_str(&self.0)
    }
}

/// One family of design tokens.
pub trait Token: Word {
    /// What follows `--` in every variant's custom property: `t-`, `s-`, `shell-`.
    const PREFIX: &'static str;
    /// Whether a consumer may write the value inline.
    const KIND: TokenKind;

    /// The custom property: `--t-tap`.
    fn var(self) -> VarName;

    /// The value in `scope`.
    fn css_value(self, scope: TokenScope) -> CssValue;

    /// The property a consumer writes to move a tuned token; `None` for a fixed one.
    fn input(self) -> Option<VarName> {
        None
    }
}
