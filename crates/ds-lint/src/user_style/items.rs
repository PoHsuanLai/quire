//! A stylesheet's structure from its flat token stream: each rule's selector, each declaration,
//! each block-less at-rule, and the faults in how they are put together. Tolerant like a browser:
//! whatever cannot be read is one fault and the walk goes on after it.

use super::note::{ParseFault, UserStyleNote, UserStyleNoteKind};
use crate::kind;
use crate::tokenize::Located;

/// One thing found in a stylesheet.
#[derive(Debug)]
pub(super) enum Part<'a> {
    /// A rule's selector: the tokens before its `{`.
    Selector(&'a [Located]),
    /// `property: value`, with the property's own token.
    Declaration {
        property: &'a Located,
        value: &'a [Located],
    },
    /// An at-rule that has no block: `@import "x";`.
    Statement(&'a [Located]),
}

/// Where a run of items sits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Level {
    /// Between rules.
    Sheet,
    /// Inside a rule's braces.
    Body,
}

/// A head and the block after it, if any.
struct Item<'a> {
    head: &'a [Located],
    block: Option<&'a [Located]>,
}

/// Every part of the sheet `tokens`, and the structural faults among them.
pub(super) fn parts(tokens: &[Located]) -> (Vec<Part<'_>>, Vec<UserStyleNote>) {
    let mut parts = Vec::new();
    let mut faults = Vec::new();
    walk(tokens, Level::Sheet, &mut parts, &mut faults);
    (parts, faults)
}

fn opens(text: &str) -> bool {
    text == "{" || text == "[" || text.ends_with('(')
}

fn closes(text: &str) -> bool {
    matches!(text, "}" | "]" | ")")
}

/// `level` split at its top-level `;` and `{ ... }` blocks.
fn items(level: &[Located]) -> Vec<Item<'_>> {
    let mut out = Vec::new();
    let (mut start, mut depth, mut block_open) = (0, 0usize, None);
    for (at, token) in level.iter().enumerate() {
        let text = token.text.as_str();
        if depth == 0 && text == ";" {
            out.push(Item {
                head: &level[start..at],
                block: None,
            });
            start = at + 1;
        } else if depth == 0 && text == "{" {
            block_open = Some(at);
            depth = 1;
        } else if opens(text) {
            depth += 1;
        } else if closes(text) && depth > 0 {
            depth -= 1;
            if depth == 0
                && let Some(open) = block_open.take()
            {
                out.push(Item {
                    head: &level[start..open],
                    block: Some(&level[open + 1..at]),
                });
                start = at + 1;
            }
        }
    }
    if start < level.len() {
        out.push(Item {
            head: &level[start..],
            block: None,
        });
    }
    out
}

fn fault(at: &Located, fault: ParseFault) -> UserStyleNote {
    UserStyleNote {
        line: at.line,
        column: at.column,
        kind: UserStyleNoteKind::Parse(fault),
    }
}

/// At-rules whose block holds more rules.
const NESTING: &[&str] = &[
    "media",
    "supports",
    "layer",
    "container",
    "scope",
    "starting-style",
    "keyframes",
];

fn walk<'a>(
    tokens: &'a [Located],
    level: Level,
    parts: &mut Vec<Part<'a>>,
    faults: &mut Vec<UserStyleNote>,
) {
    for item in items(tokens) {
        let head = kind::trim_trivia(item.head);
        let Some(first) = head.first() else { continue };
        if closes(&first.text) {
            // Reported by the token scan as a closer that closes nothing.
            continue;
        }
        let at_rule = first.text.strip_prefix('@').map(str::to_ascii_lowercase);
        match (item.block, at_rule) {
            (Some(block), Some(name)) if NESTING.contains(&name.as_str()) => {
                walk(block, Level::Sheet, parts, faults);
            }
            (Some(block), Some(_)) => walk(block, Level::Body, parts, faults),
            (Some(block), None) => {
                parts.push(Part::Selector(head));
                walk(block, Level::Body, parts, faults);
            }
            (None, Some(_)) => parts.push(Part::Statement(head)),
            (None, None) if level == Level::Sheet => {
                faults.push(fault(first, ParseFault::RuleWithoutBlock));
            }
            (None, None) => declaration(head, parts, faults),
        }
    }
}

fn declaration<'a>(
    head: &'a [Located],
    parts: &mut Vec<Part<'a>>,
    faults: &mut Vec<UserStyleNote>,
) {
    let colon = head.iter().position(|token| token.text == ":");
    let property = colon.and_then(|colon| kind::next_significant(&head[..colon], 0));
    match (colon, property) {
        (Some(colon), Some(property)) => parts.push(Part::Declaration {
            property,
            value: kind::trim_trivia(&head[colon + 1..]),
        }),
        _ => faults.push(fault(&head[0], ParseFault::DeclarationWithoutColon)),
    }
}
