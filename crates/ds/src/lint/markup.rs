//! Linting rendered markup: every class on an element is one quire exports or one
//! `consumer_css` styles, and no raw `button`/`input`/menu markup appears outside quire
//! components (coherence rule 2).
//!
//! `consumer_css` is whatever CSS the caller has in scope for the page under test — in
//! practice `ds::stylesheet()` and the consumer's own component CSS concatenated, the same way
//! a consumer's own compiled stylesheet already is (mirroring how `mail-app`'s
//! `a_class_with_no_rule_is_named` test passed its whole `STYLE` constant, tokens and
//! components together, to `unstyled_classes`). `markup` does not add `crate::stylesheet()`
//! itself, so a page that forgets to inject quire's sheet is caught too.
//!
//! A class no rule styles is [`Rule::UnstyledClass`]; an element quire draws for you written by
//! hand (an `<svg>` that is neither a `Glyph` nor marked `data-ds-svg`, a form control with no
//! `ds-` class) is [`Rule::RawMarkup`]; inline colours and durations are the stylesheet rules
//! (`super::inline_style`), except in the custom properties a quire component computes and
//! writes on its own element. Each offence's
//! selector is the element as `tag.class.class`, which is what an [`super::Exception`] names.

use std::collections::HashSet;

use super::inline_style::{self, Owner};
use super::kind;
use super::rule::{LintConfig, Offence, Rule};
use super::tokenize::{self, Located};

/// Form controls quire's components render; one without a `ds-` class was written by hand.
const CONTROLS: &[&str] = &["button", "input", "select", "textarea"];

/// Every offence in `html`, given the consumer's own stylesheet, that none of
/// `config.exceptions` covers.
pub fn markup(html: &str, consumer_css: &str, config: &LintConfig) -> Vec<Offence> {
    config.partition(every_offence(html, consumer_css)).0
}

/// Every offence in `html`, exceptions not applied.
fn every_offence(html: &str, consumer_css: &str) -> Vec<Offence> {
    let defined = defined_classes(consumer_css);
    let mut offences = Vec::new();
    let mut cursor = 0usize;
    while let Some(start) = find_tag_start(html, cursor) {
        let Some(end) = tag_end(html, start) else {
            break;
        };
        let tag = &html[start..end];
        let (line, column) = line_col_at(html, start);
        check_tag(tag, line, column, &defined, &mut offences);
        cursor = end;
    }
    offences
}

fn check_tag(tag: &str, line: u32, column: u32, defined: &HashSet<String>, out: &mut Vec<Offence>) {
    let name = tag_name(tag);
    let classes: Vec<&str> = attr_value(tag, "class")
        .map(|(value, _)| value.split_whitespace().collect())
        .unwrap_or_default();
    let selector: String = std::iter::once(name.to_ascii_lowercase())
        .chain(classes.iter().map(|class| format!(".{class}")))
        .collect();
    let mut push = |rule: Rule, line: u32, column: u32, text: String| {
        out.push(Offence {
            rule,
            selector: selector.clone(),
            line,
            column,
            text,
        });
    };
    for class in classes.iter().filter(|class| !defined.contains(**class)) {
        push(
            Rule::UnstyledClass,
            line,
            column,
            format!("<{name}>: class not defined by any rule: {class}"),
        );
    }
    let marked = attr_value(tag, "data-ds-svg").is_some();
    if let Some(what) = raw_element(name, &classes, marked) {
        push(Rule::RawMarkup, line, column, format!("<{name}>: {what}"));
    }
    if let Some((style_value, value_offset)) = attr_value(tag, "style") {
        let (local_line, local_column) = line_col_at(tag, value_offset);
        let (style_line, style_column) = if local_line == 1 {
            (line, column + local_column - 1)
        } else {
            (line + local_line - 1, local_column)
        };
        let owner = if classes.iter().any(|class| is_quire_class(class)) {
            Owner::Quire
        } else {
            Owner::Consumer
        };
        for (rule, line, column, text) in
            inline_style::offences(style_value, owner, style_line, style_column)
        {
            push(rule, line, column, text);
        }
    }
}

/// Whether `class` is one quire's components write: the root's `ds` or any `ds-*`.
fn is_quire_class(class: &str) -> bool {
    class == "ds" || class.starts_with("ds-")
}

/// Why `name` with `classes` is hand-written markup quire should have drawn, if it is. An
/// `<svg>` is quire's when it is a `Glyph` (`.ds-ic`) or carries the `data-ds-svg` marker a
/// component that draws its own vector (the SendPill's ring) writes.
fn raw_element(name: &str, classes: &[&str], marked: bool) -> Option<&'static str> {
    if name.eq_ignore_ascii_case("svg") {
        return (!classes.contains(&"ds-ic") && !marked)
            .then_some("raw svg, not .ds-ic: use Glyph instead of writing SVG markup directly");
    }
    let control = CONTROLS
        .iter()
        .any(|control| name.eq_ignore_ascii_case(control));
    let from_quire = classes.iter().any(|class| class.starts_with("ds-"));
    (control && !from_quire).then_some("a raw form control: use the quire component that draws it")
}

/// Every class a `.` selector in `css` names, anywhere (as generous as mailo's own
/// `unstyled_classes`: it does not need to know a class is reachable, only that it exists).
fn defined_classes(css: &str) -> HashSet<String> {
    let tokens: Vec<Located> = tokenize::tokens(css)
        .into_iter()
        .filter(|token| !kind::is_trivial(&token.text))
        .collect();
    let mut classes = HashSet::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.text == "."
            && let Some(next) = tokens.get(index + 1)
            && kind::is_ident(&next.text)
        {
            classes.insert(next.text.clone());
        }
    }
    classes
}

/// The byte offset of the next `<` that starts an opening tag (not `</...>` or `<!...>`),
/// at or after `from`.
fn find_tag_start(html: &str, from: usize) -> Option<usize> {
    let mut index = from;
    let bytes = html.as_bytes();
    while index < bytes.len() {
        if bytes[index] == b'<' {
            if html[index..].starts_with("</") || html[index..].starts_with("<!") {
                index = tag_end(html, index).unwrap_or(html.len());
                continue;
            }
            return Some(index);
        }
        index += 1;
    }
    None
}

/// The byte offset just past the `>` that closes the tag starting at `start`, respecting
/// quoted attribute values (which may themselves contain `>`).
fn tag_end(html: &str, start: usize) -> Option<usize> {
    let bytes = html.as_bytes();
    let mut index = start + 1;
    let mut quote: Option<u8> = None;
    while index < bytes.len() {
        let byte = bytes[index];
        match quote {
            Some(q) if byte == q => quote = None,
            Some(_) => {}
            None if byte == b'"' || byte == b'\'' => quote = Some(byte),
            None if byte == b'>' => return Some(index + 1),
            None => {}
        }
        index += 1;
    }
    None
}

fn tag_name(tag: &str) -> &str {
    let body = tag.trim_start_matches('<');
    let end = body
        .find(|c: char| c.is_whitespace() || c == '/' || c == '>')
        .unwrap_or(body.len());
    &body[..end]
}

/// `attr`'s value inside `tag`, and the byte offset of the value's first character within
/// `tag`. Matches the attribute name at a token boundary (preceded by whitespace, since a tag
/// never opens with an attribute), never as a suffix of a longer name (`data-class=` does not
/// match `class`).
fn attr_value<'a>(tag: &'a str, attr: &str) -> Option<(&'a str, usize)> {
    let lower = tag.to_ascii_lowercase();
    let needle = format!("{attr}=");
    let mut search_from = 0;
    while let Some(relative) = lower[search_from..].find(&needle) {
        let at = search_from + relative;
        let boundary = at > 0 && tag.as_bytes()[at - 1].is_ascii_whitespace();
        if boundary {
            let value_start = at + needle.len();
            let bytes = tag.as_bytes();
            if let Some(&quote) = bytes.get(value_start)
                && (quote == b'"' || quote == b'\'')
            {
                let quote = quote as char;
                if let Some(len) = tag[value_start + 1..].find(quote) {
                    return Some((
                        &tag[value_start + 1..value_start + 1 + len],
                        value_start + 1,
                    ));
                }
            }
        }
        search_from = at + needle.len();
    }
    None
}

fn line_col_at(text: &str, byte_offset: usize) -> (u32, u32) {
    let mut line = 1u32;
    let mut column = 1u32;
    for ch in text[..byte_offset.min(text.len())].chars() {
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
}
