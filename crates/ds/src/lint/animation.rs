//! `Rule::UnknownAnimation`: the keyframes an `animation-name` or an `animation` shorthand
//! names must be ones an [`crate::Anim`] plays. The shorthand was read only as
//! `animation-name` before (mailo gaps 3), so `animation: sparkle 1s` passed; now each of its
//! comma-separated animations is parsed for its name: the first identifier that is not one of
//! the shorthand's keywords (an easing, an iteration count, a direction, a fill mode, a play
//! state, a CSS-wide keyword), with times, numbers and functions (`var()`, `cubic-bezier()`,
//! `steps()`) skipped. A name spelled as a string, or an animation whose name only a `var()`
//! knows, is left alone: the lint cannot see it.

use super::declaration::push;
use super::kind;
use super::registry;
use super::rule::{Offence, Rule};
use super::tokenize::Located;
use super::walk::Decl;

/// Every keyword the `animation` shorthand accepts that is not a keyframes name. `none` is here
/// too: as a name it means no animation, which is always known.
const KEYWORDS: &[&str] = &[
    "ease",
    "ease-in",
    "ease-out",
    "ease-in-out",
    "linear",
    "step-start",
    "step-end",
    "infinite",
    "normal",
    "reverse",
    "alternate",
    "alternate-reverse",
    "none",
    "forwards",
    "backwards",
    "both",
    "running",
    "paused",
    "initial",
    "inherit",
    "unset",
    "revert",
    "revert-layer",
];

/// Which property a declaration names its animations with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Naming {
    /// `animation-name`: each comma-separated value is a name.
    Longhand,
    /// `animation`: each comma-separated value holds a name among its other parts.
    Shorthand,
}

impl Naming {
    /// The naming `property` uses, if it names animations at all.
    pub(super) fn of(property: &str) -> Option<Self> {
        match property {
            "animation-name" => Some(Naming::Longhand),
            "animation" => Some(Naming::Shorthand),
            _ => None,
        }
    }
}

/// An [`Rule::UnknownAnimation`] offence for every name in `decl` no `Anim` plays.
pub(super) fn unknown_animation(
    naming: Naming,
    selector: &str,
    decl: &Decl,
    out: &mut Vec<Offence>,
) {
    for segment in top_level(&decl.value) {
        let name = match naming {
            Naming::Longhand => kind::next_significant(segment, 0),
            Naming::Shorthand => shorthand_name(segment),
        };
        let Some(name) = name.filter(|name| kind::is_ident(&name.text)) else {
            continue;
        };
        if !registry::is_known_anim(&name.text) {
            push(out, Rule::UnknownAnimation, name, selector, &name.text);
        }
    }
}

/// `value` split at its top-level commas: the one inside `cubic-bezier(…)` does not split it.
fn top_level(value: &[Located]) -> Vec<&[Located]> {
    let mut segments = Vec::new();
    let (mut depth, mut start) = (0usize, 0usize);
    for (at, token) in value.iter().enumerate() {
        match token.text.as_str() {
            ")" | "]" => depth = depth.saturating_sub(1),
            "," if depth == 0 => {
                segments.push(&value[start..at]);
                start = at + 1;
            }
            text if opens(text) => depth += 1,
            _ => {}
        }
    }
    segments.push(&value[start..]);
    segments
}

/// Whether `text` opens a nested block: a function, a bare `(`, a `[`.
fn opens(text: &str) -> bool {
    text.ends_with('(') || text == "["
}

/// The keyframes name in one animation of a shorthand: its first top-level identifier that is
/// not a keyword. `None` when it has none (a name in a string, or only a `var()`).
fn shorthand_name(segment: &[Located]) -> Option<&Located> {
    let mut depth = 0usize;
    for token in segment {
        let text = token.text.as_str();
        if opens(text) {
            depth += 1;
            continue;
        }
        if text == ")" || text == "]" {
            depth = depth.saturating_sub(1);
            continue;
        }
        if depth > 0 || kind::is_trivial(text) || !kind::is_ident(text) {
            continue;
        }
        if !KEYWORDS.contains(&text.to_ascii_lowercase().as_str()) {
            return Some(token);
        }
    }
    None
}
