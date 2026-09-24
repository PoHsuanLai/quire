//! [`Rule::RawHairline`]: a literal hairline width (`1px`, `.5px`) in a border, an outline or a
//! one-pixel-thick box. At a fractional scale such a line covers 1.25 to 1.75 device pixels and
//! blurs; `var(--hair)` (a 1 px line) and `var(--hairline)` (a .5 px one) are whole device
//! pixels at every scale (design/01-LAYOUT.md section 2.1).

use super::declaration::push;
use super::kind;
use super::rule::{Offence, Rule};
use super::walk::Decl;

/// The border properties that carry no width.
const WIDTHLESS: &[&str] = &[
    "-radius",
    "-color",
    "-style",
    "-collapse",
    "-spacing",
    "-image",
];

/// The properties that size a box, where a one pixel value is a line drawn as a box.
const EXTENTS: &[&str] = &[
    "width",
    "height",
    "min-width",
    "min-height",
    "max-width",
    "max-height",
    "inline-size",
    "block-size",
];

/// Every hairline literal in `decl`, if `property` draws a line.
pub(super) fn raw_hairline(selector: &str, property: &str, decl: &Decl, out: &mut Vec<Offence>) {
    let lines = if draws_a_line(property) {
        decl.value.iter().collect()
    } else if EXTENTS.contains(&property) {
        let significant: Vec<_> = decl
            .value
            .iter()
            .filter(|token| !kind::is_trivial(&token.text))
            .collect();
        match significant[..] {
            [only] => vec![only],
            _ => Vec::new(),
        }
    } else {
        Vec::new()
    };
    for token in lines.into_iter().filter(|token| is_hairline(&token.text)) {
        let hint = match pixels(&token.text) {
            Some(length) if length < 1.0 => "var(--hairline)",
            _ => "var(--hair)",
        };
        let excerpt = format!("{property}: {} (use {hint})", token.text);
        push(out, Rule::RawHairline, token, selector, &excerpt);
    }
}

/// Whether `property` is a border or outline that has a width.
fn draws_a_line(property: &str) -> bool {
    let border = property.starts_with("border")
        && !WIDTHLESS.iter().any(|suffix| property.ends_with(suffix));
    border || property == "outline" || property == "outline-width"
}

/// A positive pixel length of at most one pixel.
fn is_hairline(text: &str) -> bool {
    pixels(text).is_some_and(|length| length > 0.0 && length <= 1.0)
}

/// The length of a `px` dimension token.
fn pixels(text: &str) -> Option<f32> {
    let unit = kind::dimension_unit(text)?;
    if !unit.eq_ignore_ascii_case("px") {
        return None;
    }
    text[..text.len() - unit.len()].parse().ok()
}
