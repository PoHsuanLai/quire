//! The notes: what each part of a stylesheet says about the person's use of the design system.

use super::items::Part;
use super::note::{UserStyleNote, UserStyleNoteKind};
use crate::kind;
use crate::selector::attribute_name;
use crate::tokenize::Located;
use ds_style::kit::KnownNames;
use std::collections::HashSet;

/// URL schemes that stay on the machine.
const LOCAL_SCHEMES: &[&str] = &["file:", "data:"];

/// Every note the `parts` earn against what `known` lets a stylesheet name.
pub(super) fn notes(parts: &[Part<'_>], known: &KnownNames) -> Vec<UserStyleNote> {
    let declared: HashSet<&str> = parts
        .iter()
        .filter_map(|part| match part {
            Part::Declaration { property, .. } if property.text.starts_with("--") => {
                Some(property.text.as_str())
            }
            _ => None,
        })
        .collect();
    let mut out = Vec::new();
    for part in parts {
        match part {
            Part::Selector(selector) => selector_notes(selector, known, &mut out),
            Part::Declaration { value, .. } => {
                important(value, &mut out);
                variables(value, known, &declared, &mut out);
                urls(value, &mut out);
            }
            Part::Statement(head) => urls(head, &mut out),
        }
    }
    out
}

fn note(at: &Located, kind: UserStyleNoteKind) -> UserStyleNote {
    UserStyleNote {
        line: at.line,
        column: at.column,
        kind,
    }
}

fn selector_notes(selector: &[Located], known: &KnownNames, out: &mut Vec<UserStyleNote>) {
    for (index, token) in selector.iter().enumerate() {
        let internal = |text: String| UserStyleNoteKind::InternalSelector { selector: text };
        if token.text == "."
            && let Some(class) = selector.get(index + 1)
            && !known.public_classes.contains(&class.text)
        {
            out.push(note(token, internal(format!(".{}", class.text))));
        } else if token.text.starts_with('#') && token.text.len() > 1 {
            out.push(note(token, internal(token.text.clone())));
        } else if token.text == "["
            && let Some((name, prefixed)) = attribute_name(selector, index)
        {
            let written = format!("[{}]", name.text);
            if !public_attribute(&name.text, known) {
                out.push(note(token, internal(written)));
            } else if !prefixed {
                out.push(note(
                    token,
                    UserStyleNoteKind::UnprefixedAttribute { selector: written },
                ));
            }
        }
    }
}

/// Whether `name` is a listed attribute, or in a listed family (`aria-*`).
fn public_attribute(name: &str, known: &KnownNames) -> bool {
    let name = name.to_ascii_lowercase();
    known
        .public_attributes
        .iter()
        .any(|listed| match listed.strip_suffix('*') {
            Some(family) => name.starts_with(family),
            None => name == *listed,
        })
}

fn important(value: &[Located], out: &mut Vec<UserStyleNote>) {
    for (index, token) in value.iter().enumerate() {
        let next = kind::next_significant(value, index + 1);
        if token.text == "!" && next.is_some_and(|t| t.text.eq_ignore_ascii_case("important")) {
            out.push(note(token, UserStyleNoteKind::Important));
        }
    }
}

fn variables(
    value: &[Located],
    known: &KnownNames,
    declared: &HashSet<&str>,
    out: &mut Vec<UserStyleNote>,
) {
    for (index, token) in value.iter().enumerate() {
        if !token.text.eq_ignore_ascii_case("var(") {
            continue;
        }
        let Some(name) = kind::next_significant(value, index + 1) else {
            continue;
        };
        if name.text.starts_with("--")
            && !known.vars.contains(&name.text)
            && !declared.contains(name.text.as_str())
        {
            out.push(note(
                name,
                UserStyleNoteKind::UnknownVariable {
                    name: name.text.clone(),
                },
            ));
        }
    }
}

fn urls(tokens: &[Located], out: &mut Vec<UserStyleNote>) {
    for (index, token) in tokens.iter().enumerate() {
        let lower = token.text.to_ascii_lowercase();
        let url = if lower == "url(" {
            kind::next_significant(tokens, index + 1).map(|s| unquote(&s.text))
        } else if lower.starts_with("url(") && lower.ends_with(')') {
            Some(token.text[4..token.text.len() - 1].trim().to_owned())
        } else if lower == "@import" {
            kind::next_significant(tokens, index + 1)
                .filter(|s| s.text.starts_with(['"', '\'']))
                .map(|s| unquote(&s.text))
        } else {
            None
        };
        if let Some(url) = url
            && !is_local(&url)
        {
            out.push(note(token, UserStyleNoteKind::NonLocalUrl { url }));
        }
    }
}

fn unquote(text: &str) -> String {
    text.trim_matches(['"', '\'']).to_owned()
}

fn is_local(url: &str) -> bool {
    let lower = url.trim().to_ascii_lowercase();
    LOCAL_SCHEMES.iter().any(|scheme| lower.starts_with(scheme))
}
