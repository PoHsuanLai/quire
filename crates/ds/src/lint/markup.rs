//! Linting rendered markup: every class on an element is one quire exports or one
//! `consumer_css` styles, and no raw `button`/`input`/menu markup appears outside quire
//! components (coherence rule 2).
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

use super::rule::{LintConfig, Offence};

/// Every offence in `html`, given the consumer's own stylesheet.
pub fn markup(html: &str, consumer_css: &str, config: &LintConfig) -> Vec<Offence> {
    todo!()
}
