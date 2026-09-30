//! Rendering a slice of [`super::tokenize::Located`] tokens back to a readable excerpt.
//!
//! Each `Located::text` is already the raw source slice for that one token (tokenize.rs's own
//! doc explains why), so as long as `tokens` still has its whitespace tokens in it, joining them
//! back to back reproduces the original source text exactly — including the descendant-combinator
//! space in a selector like `.presets button`, which a caller must be able to tell apart from
//! `.presets.button`. Callers that do not need that (a declaration's value, an at-rule's name)
//! get equally readable text from the same join.

use super::tokenize::Located;

/// `tokens`' own text, concatenated in order.
pub fn render(tokens: &[Located]) -> String {
    tokens.iter().map(|token| token.text.as_str()).collect()
}
