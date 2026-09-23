//! Linting rendered markup: every class on an element is one quire exports or one
//! `consumer_css` styles, and no raw `button`/`input`/menu markup appears outside quire
//! components (coherence rule 2).
//!
//! `consumer_css` is whatever CSS the caller has in scope for the page under test — in
//! practice `ds::stylesheet()` and the consumer's own component CSS concatenated, the same way
//! a consumer's own compiled stylesheet already is (mirroring how `mail-app`'s
//! `a_class_with_no_rule_is_named` test passed its whole `STYLE` constant, tokens and
//! components together, to `unstyled_classes`). `markup` does not read `crate::css::stylesheet`
//! itself: at wave 1, before the tokens crate lands its bodies, that function is `todo!()`, and
//! a lint helper that panics whenever it is asked to check markup would defeat the point of a
//! coherence check every consumer runs standalone.
//!
//! There is no `Rule` variant for "this class has no rule" or "this `<svg>` did not come from
//! `Glyph`" (the frozen `Rule` enum has one for every *stylesheet* shape, not for a markup
//! shape); both are reported as [`Rule::DsInternals`] with the real finding spelled out in the
//! offence's `text`, and CONVENTIONS' "stop and report" applies — see the worktree's report.

use std::collections::HashSet;

use super::colours;
use super::declaration::COLOUR_FUNCTIONS;
use super::kind;
use super::rule::{LintConfig, Offence, Rule};
use super::tokenize::{self, Located};

/// Every offence in `html`, given the consumer's own stylesheet.
pub fn markup(html: &str, consumer_css: &str, config: &LintConfig) -> Vec<Offence> {
    let defined = defined_classes(consumer_css);
    let mut offences = Vec::new();
    let mut cursor = 0usize;
    while let Some(start) = find_tag_start(html, cursor) {
        let Some(end) = tag_end(html, start) else {
            break;
        };
        let tag = &html[start..end];
        let (line, column) = line_col_at(html, start);
        check_tag(tag, line, column, &defined, config, &mut offences);
        cursor = end;
    }
    offences
}

fn check_tag(
    tag: &str,
    line: u32,
    column: u32,
    defined: &HashSet<String>,
    _config: &LintConfig,
    out: &mut Vec<Offence>,
) {
    let name = tag_name(tag);
    if let Some((class_value, _)) = attr_value(tag, "class") {
        for class in class_value.split_whitespace() {
            if !defined.contains(class) {
                out.push(Offence {
                    rule: Rule::DsInternals,
                    line,
                    column,
                    text: format!("<{name}>: class not defined by any rule: {class}"),
                });
            }
        }
    }
    if name.eq_ignore_ascii_case("svg") {
        let is_icon = attr_value(tag, "class")
            .is_some_and(|(value, _)| value.split_whitespace().any(|class| class == "ds-ic"));
        if !is_icon {
            out.push(Offence {
                rule: Rule::DsInternals,
                line,
                column,
                text: "raw <svg>, not .ds-ic: use Glyph instead of writing SVG markup directly"
                    .to_owned(),
            });
        }
    }
    if let Some((style_value, value_offset)) = attr_value(tag, "style") {
        let (local_line, local_column) = line_col_at(tag, value_offset);
        let (style_line, style_column) = if local_line == 1 {
            (line, column + local_column - 1)
        } else {
            (line + local_line - 1, local_column)
        };
        style_offences(style_value, style_line, style_column, out);
    }
}

fn style_offences(value: &str, base_line: u32, base_column: u32, out: &mut Vec<Offence>) {
    let tokens: Vec<Located> = tokenize::tokens(value)
        .into_iter()
        .filter(|token| !kind::is_trivial(&token.text))
        .collect();
    for token in &tokens {
        let (line, column) = if token.line == 1 {
            (base_line, base_column + token.column - 1)
        } else {
            (base_line + token.line - 1, token.column)
        };
        let rule = if kind::is_hash(&token.text) {
            Some(Rule::HexColour)
        } else if let Some(name) = kind::function_name(&token.text) {
            COLOUR_FUNCTIONS
                .contains(&name.to_ascii_lowercase().as_str())
                .then_some(Rule::ColourFunction)
        } else if kind::is_ident(&token.text) && !token.text.eq_ignore_ascii_case("currentcolor") {
            colours::is_named_colour(&token.text).then_some(Rule::NamedColour)
        } else {
            kind::dimension_unit(&token.text).and_then(|unit| {
                (unit.eq_ignore_ascii_case("ms") || unit.eq_ignore_ascii_case("s"))
                    .then_some(Rule::RawDuration)
            })
        };
        if let Some(rule) = rule {
            out.push(Offence {
                rule,
                line,
                column,
                text: format!("style=\"...\": {}", token.text),
            });
        }
    }
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
