//! The `.ds` token blocks, written from the families alone: `.ds` (light, Standard, System
//! typeface), then only what differs under `.ds[data-theme=dark]`, `.ds[data-typeface=editorial]`
//! and each non-standard `.ds[data-motion=..]`, so every name a variant block sets is also
//! declared on `.ds` (tests/tokens.rs).

use crate::core::word::Word;
use crate::style::appearance::{motion::MotionLevel, theme::Scheme, typeface::Typeface};
use crate::style::emit::{attr_selector, declaration, rule};
use crate::style::tokens::{set::TokenSet, token::TokenScope};

/// The blocks for `sets`, in the order given.
pub(super) fn token_blocks(sets: &[TokenSet]) -> String {
    let declared = |scope: TokenScope| -> Vec<String> {
        sets.iter()
            .flat_map(|set| set.declarations(scope))
            .map(|(var, value)| declaration(var, &value))
            .collect()
    };
    let base = declared(TokenScope::BASE);
    let mut light = base.clone();
    light.push("color-scheme:light;".to_owned());
    let mut dark = changed(&base, declared(TokenScope::BASE.in_scheme(Scheme::Dark)));
    dark.push("color-scheme:dark;".to_owned());
    let mut css = rule(".ds", &light);
    css.push_str(&rule(
        &format!(".ds{}", attr_selector("data-theme", Scheme::Dark.slug())),
        &dark,
    ));
    css.push_str(&rule(
        &format!(
            ".ds{}",
            attr_selector("data-typeface", Typeface::Editorial.slug())
        ),
        &changed(
            &base,
            declared(TokenScope::BASE.in_typeface(Typeface::Editorial)),
        ),
    ));
    for level in [MotionLevel::Calm, MotionLevel::Extra, MotionLevel::Reduced] {
        css.push_str(&rule(
            &format!(".ds{}", attr_selector("data-motion", level.slug())),
            &changed(&base, declared(TokenScope::BASE.at(level))),
        ));
    }
    css
}

/// The declarations of `next` that are not in `base` verbatim.
fn changed(base: &[String], next: Vec<String>) -> Vec<String> {
    next.into_iter()
        .filter(|declaration| !base.contains(declaration))
        .collect()
}
