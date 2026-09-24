//! Rules that look at one declaration's property and value: every colour, duration, easing,
//! font, geometry, var-reference, `!important` and Blitz-unsupported check.

use std::collections::HashSet;

use super::blitz;
use super::colours;
use super::kind;
use super::registry;
use super::rule::{Offence, Profile, Rule};
use super::text::render;
use super::tokenize::Located;
use super::walk::Decl;
use crate::tokens::Family;

/// Colour functions: `rgb()`, `rgba()`, `hsl()`, `hwb()`, `oklch()`, `color-mix()`, ...
pub(super) const COLOUR_FUNCTIONS: &[&str] = &[
    "rgb",
    "rgba",
    "hsl",
    "hsla",
    "hwb",
    "oklch",
    "oklab",
    "lab",
    "lch",
    "color",
    "color-mix",
];

/// The properties a raw duration or a raw easing is actually reachable from: scoping the check
/// to these properties, rather than any `Nms`/`Ns` token or bare `ease*` ident anywhere, is the
/// CONVENTIONS "ask what the protocol already told you" rule applied to CSS — a property name
/// is context we already have, so use it instead of a wider match that could catch an unrelated
/// value that merely looks the same.
const DURATION_PROPERTIES: &[&str] = &[
    "animation",
    "animation-duration",
    "animation-delay",
    "transition",
    "transition-duration",
    "transition-delay",
];

const EASING_PROPERTIES: &[&str] = &[
    "animation",
    "animation-timing-function",
    "transition",
    "transition-timing-function",
];

const EASING_IDENTS: &[&str] = &["ease", "ease-in", "ease-out", "ease-in-out", "linear"];
const EASING_FUNCTIONS: &[&str] = &["cubic-bezier", "steps", "linear"];

/// Every declaration-level offence `decl` causes, given the rule's rendered `selector`, whether
/// that selector [`super::selector::targets_svg_children`], the full set of var names the
/// stylesheet may reference without triggering [`Rule::UndeclaredVar`], and how strict the run
/// is.
pub fn offences(
    selector: &str,
    targets_svg: bool,
    decl: &Decl,
    known_vars: &HashSet<String>,
    profile: Profile,
) -> Vec<Offence> {
    let mut out = Vec::new();
    let property = decl.property.text.to_ascii_lowercase();
    let is_custom_property = property.starts_with("--");
    let stroke_or_fill = property == "stroke" || property == "fill";

    if property == "font-family" && !is_face_reference(&decl.value) {
        push(
            &mut out,
            Rule::FontFamily,
            &decl.property,
            selector,
            &decl.property.text,
        );
    }
    if stroke_or_fill && targets_svg {
        push(
            &mut out,
            Rule::SvgPaintInCss,
            &decl.property,
            selector,
            &property,
        );
    }
    if property == "animation-name" {
        unknown_animation(selector, decl, &mut out);
    }

    let strict = profile == Profile::Strict;
    if strict && !is_custom_property {
        raw_geometry(selector, &property, decl, &mut out);
        super::hairline::raw_hairline(selector, &property, decl, &mut out);
    }

    let duration_property = DURATION_PROPERTIES.contains(&property.as_str());
    let easing_property = EASING_PROPERTIES.contains(&property.as_str());

    if !is_custom_property {
        for (index, token) in decl.value.iter().enumerate() {
            value_token(
                selector,
                stroke_or_fill,
                duration_property,
                easing_property,
                &decl.value,
                index,
                token,
                known_vars,
                &mut out,
            );
        }
    }

    let value_text = render(&decl.value);
    if let Some(reason) = blitz::unsupported(&property, &value_text) {
        push(
            &mut out,
            Rule::BlitzUnsupported,
            &decl.property,
            selector,
            &format!("{}: {value_text} ({reason})", decl.property.text),
        );
    }

    out
}

#[allow(clippy::too_many_arguments)]
fn value_token(
    selector: &str,
    stroke_or_fill: bool,
    duration_property: bool,
    easing_property: bool,
    value: &[Located],
    index: usize,
    token: &Located,
    known_vars: &HashSet<String>,
    out: &mut Vec<Offence>,
) {
    let text = &token.text;
    if kind::is_hash(text) {
        push(out, Rule::HexColour, token, selector, text);
        return;
    }
    if let Some(name) = kind::function_name(text) {
        let lower = name.to_ascii_lowercase();
        if COLOUR_FUNCTIONS.contains(&lower.as_str()) {
            push(out, Rule::ColourFunction, token, selector, text);
        } else if easing_property && EASING_FUNCTIONS.contains(&lower.as_str()) {
            push(out, Rule::RawEasing, token, selector, text);
        } else if lower == "var" {
            undeclared_var(selector, value, index, known_vars, out);
        }
        return;
    }
    if kind::is_ident(text) {
        if text.eq_ignore_ascii_case("currentcolor") {
            let whole_value = value.iter().filter(|t| !kind::is_trivial(&t.text)).count() == 1;
            if !(stroke_or_fill && whole_value) {
                push(
                    out,
                    Rule::CurrentColourOutsideStrokeFill,
                    token,
                    selector,
                    text,
                );
            }
        } else if colours::is_named_colour(text) {
            push(out, Rule::NamedColour, token, selector, text);
        } else if easing_property && EASING_IDENTS.contains(&text.to_ascii_lowercase().as_str()) {
            push(out, Rule::RawEasing, token, selector, text);
        }
        return;
    }
    if text == "!" {
        if let Some(next) = kind::next_significant(value, index + 1)
            && next.text.eq_ignore_ascii_case("important")
        {
            push(out, Rule::Important, token, selector, "!important");
        }
        return;
    }
    if duration_property
        && let Some(unit) = kind::dimension_unit(text)
        && (unit.eq_ignore_ascii_case("ms") || unit.eq_ignore_ascii_case("s"))
    {
        push(out, Rule::RawDuration, token, selector, text);
    }
}

fn undeclared_var(
    selector: &str,
    value: &[Located],
    var_index: usize,
    known_vars: &HashSet<String>,
    out: &mut Vec<Offence>,
) {
    let Some(name) = kind::next_significant(value, var_index + 1) else {
        return;
    };
    if !name.text.starts_with("--") {
        return;
    }
    if !known_vars.contains(&name.text) {
        push(out, Rule::UndeclaredVar, name, selector, &name.text);
    }
}

fn unknown_animation(selector: &str, decl: &Decl, out: &mut Vec<Offence>) {
    for segment in decl.value.split(|t| t.text == ",") {
        let Some(name) = kind::next_significant(segment, 0) else {
            continue;
        };
        if !kind::is_ident(&name.text) {
            continue;
        }
        if !registry::is_known_anim(&name.text) {
            push(out, Rule::UnknownAnimation, name, selector, &name.text);
        }
    }
}

/// Whether a `font-family` value is one of the face tokens (`var(--font-ui)`) or `inherit`:
/// the value the rule asks for, so it is not an offence.
fn is_face_reference(value: &[Located]) -> bool {
    let significant: Vec<&str> = value
        .iter()
        .map(|token| token.text.as_str())
        .filter(|text| !kind::is_trivial(text))
        .collect();
    match significant[..] {
        ["inherit"] => true,
        ["var(", name, ")"] => [Family::Display, Family::Ui, Family::Data]
            .iter()
            .any(|family| family.var().as_str() == name),
        _ => false,
    }
}

/// Whether `property` spaces things out: `margin`, `padding` and their sides and logical
/// forms, and the three gaps.
fn is_spacing_property(property: &str) -> bool {
    ["margin", "padding"].iter().any(|base| {
        property == *base
            || property
                .strip_prefix(base)
                .is_some_and(|rest| rest.starts_with('-'))
    }) || matches!(property, "gap" | "row-gap" | "column-gap")
}

fn raw_geometry(selector: &str, property: &str, decl: &Decl, out: &mut Vec<Offence>) {
    let rule = if property == "z-index" {
        Rule::RawZIndex
    } else if property == "font-size" {
        Rule::RawFontSize
    } else if property == "border-radius" || property.ends_with("-radius") {
        Rule::RawRadius
    } else if is_spacing_property(property) {
        raw_spacing(selector, decl, out);
        return;
    } else {
        return;
    };
    for token in &decl.value {
        // A square corner is no radius at all; there is nothing for a token to name.
        if rule == Rule::RawRadius && token.text == "0" {
            continue;
        }
        let raw = kind::is_number(&token.text)
            || kind::is_percentage(&token.text)
            || kind::dimension_unit(&token.text).is_some();
        if raw {
            push(out, rule, token, selector, &token.text);
        }
    }
}

/// A literal length in pixels is the offence; `0`, `auto`, a percentage, an `em` and a
/// `var(--s-*)` are not (the scale is in pixels, so only a pixel literal bypasses it).
fn raw_spacing(selector: &str, decl: &Decl, out: &mut Vec<Offence>) {
    for token in &decl.value {
        if kind::dimension_unit(&token.text).is_some_and(|unit| unit.eq_ignore_ascii_case("px")) {
            push(out, Rule::RawSpacing, token, selector, &token.text);
        }
    }
}

pub(super) fn push(
    out: &mut Vec<Offence>,
    rule: Rule,
    at: &Located,
    selector: &str,
    excerpt: &str,
) {
    let text = if selector.is_empty() {
        excerpt.to_owned()
    } else {
        format!("{selector}: {excerpt}")
    };
    out.push(Offence {
        rule,
        selector: selector.to_owned(),
        line: at.line,
        column: at.column,
        text,
    });
}
