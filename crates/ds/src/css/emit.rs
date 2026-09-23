//! Writing CSS text: declarations, rules, custom properties. Shared by every generator.
//!
//! Every attribute selector is written with the `*|` namespace, `[*|data-theme=dark]`:
//! dioxus-native-dom puts attributes in the HTML namespace, where an unprefixed attribute
//! selector never matches; browsers accept the `*|` form too (spike S2). Class selectors are
//! unaffected.
//!
//! The output is one rule per line and no space inside a rule, so the golden stylesheet diffs
//! by rule.

use crate::tokens::VarName;

/// A custom property declaration: `--t-tap:90ms;`.
pub fn declaration(var: VarName, value: &str) -> String {
    format!("{}:{value};", var.as_str())
}

/// A rule: `selector{declarations}`.
pub fn rule(selector: &str, declarations: &[String]) -> String {
    format!("{selector}{{{}}}\n", declarations.concat())
}

/// An attribute selector in the form Blitz matches: `[*|data-theme=dark]`.
pub fn attr_selector(attribute: &str, value: &str) -> String {
    format!("[*|{attribute}={value}]")
}

/// A presence selector in the form Blitz matches: `[*|data-material]`.
pub(crate) fn presence_selector(attribute: &str) -> String {
    format!("[*|{attribute}]")
}

/// A plain (non-custom) declaration, `animation:gulp …;`, for the few rules the generators
/// write that are not token blocks.
pub(crate) fn property(name: &str, value: &str) -> String {
    format!("{name}:{value};")
}

#[cfg(test)]
mod tests {
    use super::{attr_selector, declaration, rule};
    use crate::tokens::VarName;

    #[test]
    fn the_pieces_compose_into_one_line_per_rule() {
        let selector = format!(".ds{}", attr_selector("data-theme", "dark"));
        let css = rule(
            &selector,
            &[
                declaration(VarName("--paper"), "#151814"),
                declaration(VarName("--t-tap"), "90ms"),
            ],
        );
        assert_eq!(
            css,
            ".ds[*|data-theme=dark]{--paper:#151814;--t-tap:90ms;}\n"
        );
    }
}
