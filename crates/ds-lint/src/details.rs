//! The details grammar's two CSS rules (design/26-DETAILS.md R3, section 3.4):
//! [`Rule::InfiniteLoop`] rejects `animation-iteration-count: infinite` and `infinite` in the
//! `animation` shorthand, in every profile; [`Rule::OffGrammarTiming`] rejects, under
//! [`Profile::Details`], an `animation` or `transition` whose duration or easing token is not one
//! the grammar plays (`ds::motion::detail::grammar`). Raw values are `RawDuration` and `RawEasing`'s.

use super::declaration::push;
use super::kind;
use super::rule::{Offence, Profile, Rule};
use super::walk::Decl;
use ds_core::word::Word;
use ds_style::kit::KnownNames;
use ds_style::tokens::{easing::EasingToken, timing::DurationToken};

/// The properties whose `var()`s time an animation or a transition.
const TIMING_PROPERTIES: &[&str] = &[
    "animation",
    "animation-duration",
    "animation-timing-function",
    "transition",
    "transition-duration",
    "transition-timing-function",
];

/// Every details offence `decl` causes under `profile`.
pub(super) fn offences(
    selector: &str,
    property: &str,
    decl: &Decl,
    known: &KnownNames,
    profile: Profile,
    out: &mut Vec<Offence>,
) {
    infinite(selector, property, decl, out);
    if profile == Profile::Details && TIMING_PROPERTIES.contains(&property) {
        off_grammar(selector, decl, known, out);
    }
}

/// `infinite` as an iteration count: the longhand's value, or an identifier at the shorthand's
/// top level (never inside a function).
fn infinite(selector: &str, property: &str, decl: &Decl, out: &mut Vec<Offence>) {
    if property != "animation" && property != "animation-iteration-count" {
        return;
    }
    let mut depth = 0usize;
    for token in &decl.value {
        let text = token.text.as_str();
        if text.ends_with('(') {
            depth += 1;
        } else if text == ")" {
            depth = depth.saturating_sub(1);
        } else if depth == 0 && kind::is_ident(text) && text.eq_ignore_ascii_case("infinite") {
            push(
                out,
                Rule::InfiniteLoop,
                token,
                selector,
                &format!("{property}: infinite"),
            );
        }
    }
}

/// A `var(--t-*)` or `var(--e-*)` naming a token outside the grammar.
fn off_grammar(selector: &str, decl: &Decl, known: &KnownNames, out: &mut Vec<Offence>) {
    for (index, token) in decl.value.iter().enumerate() {
        if !token.text.eq_ignore_ascii_case("var(") {
            continue;
        }
        let Some(name) = kind::next_significant(&decl.value, index + 1) else {
            continue;
        };
        if outside_grammar(&name.text, known) {
            push(
                out,
                Rule::OffGrammarTiming,
                name,
                selector,
                &format!("{} is not a grammar token", name.text),
            );
        }
    }
}

/// Whether `var_name` is a duration or easing token the grammar does not play.
fn outside_grammar(var_name: &str, known: &KnownNames) -> bool {
    let duration = DurationToken::ALL
        .iter()
        .find(|token| token.var().as_str() == var_name)
        .is_some_and(|token| !known.grammar_durations.contains(token));
    let easing = EasingToken::ALL
        .iter()
        .find(|token| token.var().as_str() == var_name)
        .is_some_and(|token| !known.grammar_easings.contains(token));
    duration || easing
}
