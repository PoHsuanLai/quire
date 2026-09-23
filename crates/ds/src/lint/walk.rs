//! Turning a flat token stream (from [`super::tokenize::tokens`], whitespace and comments
//! still in it) into rules and their declarations, one pass, by tracking `{`/`}` depth — the
//! same shape as `mailo`'s old `component_rules`, but over tokens instead of characters.
//! Whitespace stays in deliberately: it is the only way to tell `.a.b` (one compound selector)
//! from `.a .b` (a descendant combinator), and [`super::text::render`] needs it to reproduce a
//! selector's own text for an offence's excerpt.

use super::kind;
use super::rule::{Offence, Rule};
use super::text::render;
use super::tokenize::Located;

/// One property/value pair inside a rule's body.
pub struct Decl {
    /// The property name token (`color`, `--my-gap`, `-webkit-line-clamp`).
    pub property: Located,
    /// The value's tokens, in order, trimmed of leading and trailing whitespace/comments
    /// (internal whitespace, as in `1px solid var(--line)`, is kept).
    pub value: Vec<Located>,
}

/// One qualified rule or at-rule-as-declarations block (`@font-face { ... }`), flattened out of
/// whatever `@media`/`@supports` nesting held it, with one `@keyframes` step counted as its own
/// rule (its "selector" is `0%`, `from`, `to`, ...).
pub struct CollectedRule {
    /// The selector (or at-rule name), rendered for offence text.
    pub selector: String,
    /// The selector's own significant tokens, for the selector-level rules.
    pub selector_tokens: Vec<Located>,
    /// Its declarations.
    pub declarations: Vec<Decl>,
}

/// Every rule in `tokens`, plus at-rule-level offences found along the way (today, only
/// [`Rule::Keyframes`]).
pub fn walk(tokens: &[Located]) -> (Vec<CollectedRule>, Vec<Offence>) {
    let mut rules = Vec::new();
    let mut offences = Vec::new();
    walk_block(tokens, &mut rules, &mut offences);
    (rules, offences)
}

fn walk_block(level: &[Located], rules: &mut Vec<CollectedRule>, offences: &mut Vec<Offence>) {
    let mut index = 0;
    while index < level.len() {
        if level[index].text == ";" {
            // A stray top-level semicolon: not a rule, skip it.
            index += 1;
            continue;
        }
        let Some(offset) = level[index..].iter().position(|t| t.text == "{") else {
            // No further rules at this level; anything left over is not well-formed CSS.
            break;
        };
        let brace = index + offset;
        let prelude = &level[index..brace];
        let body_start = brace + 1;
        let body_end = matching_close(level, body_start);
        let body = &level[body_start..body_end.min(level.len())];
        handle_rule(prelude, body, rules, offences);
        index = body_end + 1;
    }
}

/// The index of the `}` that closes the block starting right after `level[start - 1] == '{'`.
fn matching_close(level: &[Located], start: usize) -> usize {
    let mut depth = 1usize;
    let mut index = start;
    while index < level.len() {
        match level[index].text.as_str() {
            "{" => depth += 1,
            "}" => {
                depth -= 1;
                if depth == 0 {
                    return index;
                }
            }
            _ => {}
        }
        index += 1;
    }
    level.len()
}

fn handle_rule(
    prelude: &[Located],
    body: &[Located],
    rules: &mut Vec<CollectedRule>,
    offences: &mut Vec<Offence>,
) {
    let prelude = kind::trim_trivia(prelude);
    if let Some(first) = prelude.first()
        && let Some(keyword) = first.text.strip_prefix('@')
    {
        match keyword.to_ascii_lowercase().as_str() {
            "keyframes" => {
                offences.push(Offence {
                    rule: Rule::Keyframes,
                    line: first.line,
                    column: first.column,
                    text: render(prelude),
                });
                // Each step (`0%`, `from`, `to`, ...) is its own nested rule.
                walk_block(body, rules, offences);
                return;
            }
            "media" | "supports" | "layer" | "container" | "scope" | "starting-style" => {
                walk_block(body, rules, offences);
                return;
            }
            _ => {
                // `@font-face`, `@page`, and anything else this design system never
                // writes: a flat declaration list, not nested rules.
                push_rule(prelude, body, rules);
                return;
            }
        }
    }
    push_rule(prelude, body, rules);
}

fn push_rule(prelude: &[Located], body: &[Located], rules: &mut Vec<CollectedRule>) {
    rules.push(CollectedRule {
        selector: render(prelude),
        selector_tokens: prelude.to_vec(),
        declarations: collect_declarations(body),
    });
}

fn collect_declarations(body: &[Located]) -> Vec<Decl> {
    let mut declarations = Vec::new();
    let mut start = 0;
    for (index, token) in body.iter().enumerate() {
        if token.text == ";" {
            push_declaration(&body[start..index], &mut declarations);
            start = index + 1;
        }
    }
    if start < body.len() {
        push_declaration(&body[start..], &mut declarations);
    }
    declarations
}

fn push_declaration(segment: &[Located], declarations: &mut Vec<Decl>) {
    let Some(colon) = segment.iter().position(|t| t.text == ":") else {
        return;
    };
    let Some(property) = kind::next_significant(&segment[..colon], 0).cloned() else {
        return;
    };
    declarations.push(Decl {
        property,
        value: kind::trim_trivia(&segment[colon + 1..]).to_vec(),
    });
}
