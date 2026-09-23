//! Writing CSS text: declarations, rules, custom properties. Shared by every generator.
//!
//! Every attribute selector is written with the `*|` namespace, `[*|data-theme=dark]`:
//! dioxus-native-dom puts attributes in the HTML namespace, where an unprefixed attribute
//! selector never matches; browsers accept the `*|` form too (spike S2). Class selectors are
//! unaffected.
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

use crate::tokens::VarName;

/// A custom property declaration: `--t-tap:90ms;`.
pub fn declaration(var: VarName, value: &str) -> String {
    todo!()
}

/// A rule: `selector{declarations}`.
pub fn rule(selector: &str, declarations: &[String]) -> String {
    todo!()
}

/// An attribute selector in the form Blitz matches: `[*|data-theme=dark]`.
pub fn attr_selector(attribute: &str, value: &str) -> String {
    todo!()
}
