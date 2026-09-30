//! The HIG guardrails on a stylesheet (design/27-HIG-PARITY.md section 7, H0), all
//! [`super::Severity::Warning`] today: [`Rule::PointerCursor`] (the pointing hand only on a
//! link, section 6.3), [`Rule::MinFontSize`] (no text under the 10 px floor, section 3.16) and
//! [`Rule::FocusRingShape`] (a focus ring's radius is its control's plus the gap, sections 3.9
//! and 6.4).

use super::declaration::push;
use super::kind;
use super::rule::{Offence, Rule};
use super::tokenize::Located;
use super::walk::{CollectedRule, Decl};
use crate::style::appearance::typeface::Typeface;
use crate::style::tokens::type_scale::FontSize;
use ds_core::word::Word;

/// The pseudo-classes that mark a focus rule (`:focus-visible` and `:focus-within` are
/// `FocusPseudoClass`'s to reject, but a ring written under them is still a ring).
const FOCUS_PSEUDOS: &[&str] = &["focus", "focus-visible", "focus-within"];

/// Every guardrail offence in `rule`.
pub(super) fn offences(rule: &CollectedRule) -> Vec<Offence> {
    let mut out = Vec::new();
    let link = subject_is_link(&rule.selector_tokens);
    let focus = focus_subject(&rule.selector_tokens);
    for decl in &rule.declarations {
        let property = decl.property.text.to_ascii_lowercase();
        if property == "cursor" && link == Subject::Other {
            pointer_cursor(&rule.selector, decl, &mut out);
        }
        if property == "font-size" || property == "font" {
            min_font_size(&rule.selector, &property, decl, &mut out);
        }
        if focus == Subject::Focus && is_radius(&property) && !is_ring_radius(&decl.value) {
            let excerpt = format!(
                "{property}: not calc(var(--r-*) + var(--focus-gap)) (a ring follows its control)"
            );
            push(
                &mut out,
                Rule::FocusRingShape,
                &decl.property,
                &rule.selector,
                &excerpt,
            );
        }
    }
    out
}

/// What a rule's selector picks out, as far as the guardrails care.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Subject {
    /// Every selector in the list ends on a link.
    Link,
    /// The selector carries a focus pseudo-class.
    Focus,
    /// Anything else.
    Other,
}

/// `cursor: pointer` outside a link.
fn pointer_cursor(selector: &str, decl: &Decl, out: &mut Vec<Offence>) {
    for token in decl
        .value
        .iter()
        .filter(|token| token.text.eq_ignore_ascii_case("pointer"))
    {
        push(
            out,
            Rule::PointerCursor,
            token,
            selector,
            "cursor: pointer on a control (the arrow, cursor: default; the hand is for links)",
        );
    }
}

/// Every size under the floor in a `font-size` or `font` value: a `px` or `pt` literal (in
/// `font`, one before the `/` of the line height), or a `var(--fs-*)` whose System value is
/// under it.
fn min_font_size(selector: &str, property: &str, decl: &Decl, out: &mut Vec<Offence>) {
    let mut after_slash = Slash::Before;
    for (index, token) in decl.value.iter().enumerate() {
        let text = token.text.as_str();
        if text == "/" {
            after_slash = Slash::After;
            continue;
        }
        if after_slash == Slash::Before
            && let Some(px) = literal_px(text)
            && px < FontSize::MIN_PX
        {
            let excerpt = format!("{property}: {text} (under the 10 px floor)");
            push(out, Rule::MinFontSize, token, selector, &excerpt);
        }
        if text.eq_ignore_ascii_case("var(")
            && let Some(name) = kind::next_significant(&decl.value, index + 1)
            && let Some(size) = small_token(&name.text)
        {
            let excerpt = format!(
                "{property}: var({}) is {} under System (under the 10 px floor)",
                name.text,
                size.css_in(Typeface::System)
            );
            push(out, Rule::MinFontSize, name, selector, &excerpt);
        }
    }
}

/// Whether a token comes before or after the `font` shorthand's `/`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Slash {
    /// Before: a size.
    Before,
    /// After: a line height.
    After,
}

/// A `px` or `pt` dimension's length in px.
fn literal_px(text: &str) -> Option<f32> {
    let unit = kind::dimension_unit(text)?.to_ascii_lowercase();
    let number: f32 = text[..text.len() - unit.len()].parse().ok()?;
    match unit.as_str() {
        "px" => Some(number),
        "pt" => Some(number * 4.0 / 3.0),
        _ => None,
    }
}

/// The size step `name` names, when it is under the floor under System.
fn small_token(name: &str) -> Option<FontSize> {
    FontSize::ALL
        .iter()
        .copied()
        .find(|size| size.var().as_str() == name && size.px_in(Typeface::System) < FontSize::MIN_PX)
}

/// Whether `property` sets a corner.
fn is_radius(property: &str) -> bool {
    property == "border-radius"
        || (property.starts_with("border-") && property.ends_with("-radius"))
}

/// Whether `value` is exactly `calc(var(--r-*) + var(--focus-gap))`.
fn is_ring_radius(value: &[Located]) -> bool {
    let significant: Vec<&str> = value
        .iter()
        .map(|token| token.text.as_str())
        .filter(|text| !kind::is_trivial(text))
        .collect();
    match significant[..] {
        [
            calc,
            "var(",
            radius,
            ")",
            "+",
            "var(",
            "--focus-gap",
            ")",
            ")",
        ] => calc.eq_ignore_ascii_case("calc(") && radius.starts_with("--r-"),
        _ => false,
    }
}

/// [`Subject::Focus`] when the selector names a focus pseudo-class (`:focus`, never the `::` of an element).
fn focus_subject(selector: &[Located]) -> Subject {
    let focus = selector.windows(2).enumerate().any(|(index, pair)| {
        let single_colon = pair[0].text == ":" && (index == 0 || selector[index - 1].text != ":");
        single_colon
            && FOCUS_PSEUDOS
                .iter()
                .any(|pseudo| pair[1].text.eq_ignore_ascii_case(pseudo))
    });
    if focus {
        Subject::Focus
    } else {
        Subject::Other
    }
}

/// [`Subject::Link`] when every selector in the list ends on a link: its last compound is an `a`, carries a
/// class with a `link` segment (`.ds-run-link`, `.ds-link-pill`), or a `role=link` attribute.
fn subject_is_link(selector: &[Located]) -> Subject {
    let parts: Vec<&[Located]> = selector.split(|token| token.text == ",").collect();
    let every = parts
        .iter()
        .all(|part| compound_is_link(last_compound(kind::trim_trivia(part))));
    if every && !parts.is_empty() {
        Subject::Link
    } else {
        Subject::Other
    }
}

/// The tokens after the last combinator (whitespace, `>`, `+` or `~`) outside brackets.
fn last_compound(part: &[Located]) -> &[Located] {
    let mut depth = 0usize;
    let mut start = 0usize;
    for (index, token) in part.iter().enumerate() {
        match token.text.as_str() {
            "[" | "(" => depth += 1,
            "]" | ")" => depth = depth.saturating_sub(1),
            text if text.ends_with('(') => depth += 1,
            ">" | "+" | "~" if depth == 0 => start = index + 1,
            text if depth == 0 && kind::is_trivial(text) => start = index + 1,
            _ => {}
        }
    }
    &part[start.min(part.len())..]
}

/// Whether one compound selector is a link.
fn compound_is_link(compound: &[Located]) -> bool {
    let anchor = compound
        .first()
        .is_some_and(|first| first.text.eq_ignore_ascii_case("a"));
    let class = compound.windows(2).any(|pair| {
        pair[0].text == "."
            && pair[1]
                .text
                .split('-')
                .any(|segment| segment.eq_ignore_ascii_case("link"))
    });
    let role = compound.windows(3).any(|triple| {
        triple[0].text.eq_ignore_ascii_case("role")
            && triple[1].text == "="
            && triple[2]
                .text
                .trim_matches(['"', '\''])
                .eq_ignore_ascii_case("link")
    });
    anchor || class || role
}
