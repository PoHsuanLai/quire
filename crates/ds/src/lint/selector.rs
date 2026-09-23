//! Rules that look at a qualified rule's selector prelude, not its declarations:
//! [`Rule::RootSelector`], [`Rule::DsInternals`], [`Rule::UnprefixedAttributeSelector`],
//! [`Rule::FocusPseudoClass`], and the "does this selector reach into an SVG's children"
//! question [`Rule::SvgPaintInCss`] asks about a `stroke`/`fill` declaration.

use super::kind;
use super::rule::{Offence, Rule};
use super::tokenize::Located;

/// The six attributes a `.ds` root carries that a consumer must never select on
/// (design/22-SETTINGS.md, CONVENTIONS §11).
const INTERNAL_ATTRS: &[&str] = &[
    "data-theme",
    "data-accent",
    "data-motion",
    "data-material",
    "data-blur",
    "data-modality",
];

/// SVG element types whose paint never reaches a stylesheet rule (FINDINGS spike S6).
const SVG_CHILD_TYPES: &[&str] = &[
    "svg", "path", "circle", "rect", "line", "polygon", "polyline", "ellipse", "g", "use",
];

/// Every selector-level offence in one rule's prelude (its significant tokens, `{` excluded).
pub fn offences(prelude: &[Located]) -> Vec<Offence> {
    let mut out = Vec::new();
    root_selector(prelude, &mut out);
    ds_internals(prelude, &mut out);
    unprefixed_attribute(prelude, &mut out);
    focus_pseudo(prelude, &mut out);
    out
}

/// Whether `prelude` selects an SVG element or one of its usual child element types
/// (a type selector, never a class that merely starts with the same letters).
pub fn targets_svg_children(prelude: &[Located]) -> bool {
    prelude.iter().enumerate().any(|(index, token)| {
        kind::is_ident(&token.text)
            && SVG_CHILD_TYPES
                .iter()
                .any(|svg| token.text.eq_ignore_ascii_case(svg))
            && !preceded_by_dot(prelude, index)
    })
}

fn preceded_by_dot(tokens: &[Located], index: usize) -> bool {
    index > 0 && tokens[index - 1].text == "."
}

fn root_selector(prelude: &[Located], out: &mut Vec<Offence>) {
    for (index, token) in prelude.iter().enumerate() {
        if token.text == ":" {
            if let Some(next) = prelude.get(index + 1)
                && next.text.eq_ignore_ascii_case("root")
            {
                out.push(Offence {
                    rule: Rule::RootSelector,
                    line: token.line,
                    column: token.column,
                    text: ":root".to_owned(),
                });
            }
            continue;
        }
        let is_html_or_body = kind::is_ident(&token.text)
            && matches!(token.text.to_ascii_lowercase().as_str(), "html" | "body");
        if is_html_or_body && !preceded_by_dot(prelude, index) {
            out.push(Offence {
                rule: Rule::RootSelector,
                line: token.line,
                column: token.column,
                text: token.text.clone(),
            });
        }
    }
}

fn ds_internals(prelude: &[Located], out: &mut Vec<Offence>) {
    for (index, token) in prelude.iter().enumerate() {
        if token.text == "." {
            if let Some(class) = prelude.get(index + 1) {
                let lower = class.text.to_ascii_lowercase();
                if lower == "ds" || lower.starts_with("ds-") {
                    out.push(Offence {
                        rule: Rule::DsInternals,
                        line: token.line,
                        column: token.column,
                        text: format!(".{}", class.text),
                    });
                }
            }
        } else if token.text == "["
            && let Some((name, _prefixed)) = attribute_name(prelude, index)
        {
            let lower = name.text.to_ascii_lowercase();
            if INTERNAL_ATTRS.contains(&lower.as_str()) {
                out.push(Offence {
                    rule: Rule::DsInternals,
                    line: token.line,
                    column: token.column,
                    text: format!("[{}]", name.text),
                });
            }
        }
    }
}

fn unprefixed_attribute(prelude: &[Located], out: &mut Vec<Offence>) {
    for (index, token) in prelude.iter().enumerate() {
        if token.text != "[" {
            continue;
        }
        let Some((name, prefixed)) = attribute_name(prelude, index) else {
            continue;
        };
        if prefixed {
            continue;
        }
        let lower = name.text.to_ascii_lowercase();
        if lower.starts_with("data-") || lower.starts_with("aria-") {
            out.push(Offence {
                rule: Rule::UnprefixedAttributeSelector,
                line: token.line,
                column: token.column,
                text: format!("[{}]", name.text),
            });
        }
    }
}

/// The attribute-name token right after `prelude[open]` (a `[`), skipping a `*|` namespace
/// prefix if there is one. `true` means the namespace prefix was present.
fn attribute_name(prelude: &[Located], open: usize) -> Option<(&Located, bool)> {
    let mut index = open + 1;
    let prefixed = prelude.get(index).map(|t| t.text.as_str()) == Some("*")
        && prelude.get(index + 1).map(|t| t.text.as_str()) == Some("|");
    if prefixed {
        index += 2;
    }
    prelude.get(index).map(|name| (name, prefixed))
}

fn focus_pseudo(prelude: &[Located], out: &mut Vec<Offence>) {
    for (index, token) in prelude.iter().enumerate() {
        if token.text != ":" {
            continue;
        }
        if let Some(next) = prelude.get(index + 1) {
            let lower = next.text.to_ascii_lowercase();
            if lower == "focus-visible" || lower == "focus-within" {
                out.push(Offence {
                    rule: Rule::FocusPseudoClass,
                    line: token.line,
                    column: token.column,
                    text: format!(":{}", next.text),
                });
            }
        }
    }
}
