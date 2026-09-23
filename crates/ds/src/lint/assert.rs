//! The one call a consumer's test makes.
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

use super::rule::LintConfig;

/// Panic, listing every offence with its line and column, unless `css` is clean.
pub fn assert_clean(css: &str, config: &LintConfig) {
    todo!()
}
