//! The cost notes: CSS that is legal but slows a frame or does nothing on this desktop
//! (DESIGN.md section 7.5). Report only.

use super::limits::{
    EM_PX, MAX_BLUR_RADIUS_PX, MAX_SHADOW_BLUR_TOTAL_PX, SPINNER_CLASSES, SURFACE_ATTRIBUTE,
};
use super::note::{UserStyleNote, UserStyleNoteKind};
use crate::kind;
use crate::tokenize::Located;

/// Every cost note for one declaration of a rule whose selector is `scope`.
pub(super) fn notes(
    property: &Located,
    value: &[Located],
    scope: &[Located],
    out: &mut Vec<UserStyleNote>,
) {
    let name = property.text.to_ascii_lowercase();
    let mut push = |kind: UserStyleNoteKind| {
        out.push(UserStyleNote {
            line: property.line,
            column: property.column,
            kind,
        });
    };
    // A property Blitz does not draw costs nothing, so it earns only the `NoEffect` note: a blur
    // that is never painted is not also a heavy one.
    let text: String = value.iter().map(|token| token.text.as_str()).collect();
    if crate::blitz::unsupported(&name, &text).is_some() {
        push(UserStyleNoteKind::NoEffect { property: name });
        return;
    }
    match name.as_str() {
        "backdrop-filter" | "filter" if heavy_blur(value, scope) => {
            push(UserStyleNoteKind::HeavyBlur {
                property: name.clone(),
            })
        }
        "box-shadow" if shadow_blur(value) > MAX_SHADOW_BLUR_TOTAL_PX => {
            push(UserStyleNoteKind::LargeShadow)
        }
        "animation" | "animation-iteration-count"
            if has_ident(value, "infinite") && !in_spinner(scope) =>
        {
            push(UserStyleNoteKind::EndlessAnimation)
        }
        "transition" | "transition-property" if has_ident(value, "all") => {
            push(UserStyleNoteKind::AnimatesEverything)
        }
        _ => {}
    }
}

fn has_ident(value: &[Located], word: &str) -> bool {
    value
        .iter()
        .any(|token| token.text.eq_ignore_ascii_case(word))
}

/// A CSS length in pixels, or `None` when it is not a plain length (`calc(`, `var(`, `50%`).
fn length_px(text: &str) -> Option<f32> {
    if kind::is_number(text) {
        return text.parse::<f32>().ok().filter(|n| *n == 0.0);
    }
    let unit = kind::dimension_unit(text)?;
    let number: f32 = text[..text.len() - unit.len()].parse().ok()?;
    match unit.to_ascii_lowercase().as_str() {
        "px" => Some(number),
        "em" | "rem" => Some(number * EM_PX),
        _ => None,
    }
}

/// Every `blur(` radius in `value`, in pixels, `None` for one that is not a plain length.
fn blur_radii(value: &[Located]) -> Vec<Option<f32>> {
    value
        .iter()
        .enumerate()
        .filter(|(_, token)| token.text.eq_ignore_ascii_case("blur("))
        .map(|(at, _)| kind::next_significant(value, at + 1).and_then(|r| length_px(&r.text)))
        .collect()
}

fn heavy_blur(value: &[Located], scope: &[Located]) -> bool {
    let radii = blur_radii(value);
    let over = radii
        .iter()
        .any(|radius| radius.is_some_and(|px| px > MAX_BLUR_RADIUS_PX));
    over || (!radii.is_empty() && on_surface_root(scope))
}

/// The blur of every layer of a `box-shadow`, added up: the third length of each layer.
fn shadow_blur(value: &[Located]) -> f32 {
    let mut layers: Vec<Vec<f32>> = vec![Vec::new()];
    let mut depth = 0usize;
    for token in value {
        let text = token.text.as_str();
        if text.ends_with('(') {
            depth += 1;
        } else if text == ")" {
            depth = depth.saturating_sub(1);
        } else if depth == 0 && text == "," {
            layers.push(Vec::new());
        } else if depth == 0
            && !kind::is_trivial(text)
            && let Some(px) = length_px(text)
            && let Some(layer) = layers.last_mut()
        {
            layer.push(px);
        }
    }
    layers
        .iter()
        .map(|lengths| lengths.get(2).copied().unwrap_or(0.0))
        .sum()
}

/// The last compound selector of each comma-separated selector in `scope`: the part the rule
/// styles. `.a .b, .c` gives `.b` and `.c`.
fn targets(scope: &[Located]) -> Vec<Vec<&Located>> {
    let mut out = Vec::new();
    let (mut depth, mut compound): (usize, Vec<&Located>) = (0, Vec::new());
    for token in scope {
        let text = token.text.as_str();
        if text.ends_with('(') || text == "[" {
            depth += 1;
        } else if matches!(text, ")" | "]") {
            depth = depth.saturating_sub(1);
        }
        match (depth, text) {
            (0, ",") => out.push(std::mem::take(&mut compound)),
            (0, ">" | "+" | "~") => compound.clear(),
            (0, _) if kind::is_trivial(text) => compound.clear(),
            _ => compound.push(token),
        }
    }
    out.push(compound);
    out
}

fn on_surface_root(scope: &[Located]) -> bool {
    targets(scope).iter().any(|compound| {
        compound
            .iter()
            .any(|token| token.text.eq_ignore_ascii_case(SURFACE_ATTRIBUTE))
    })
}

fn in_spinner(scope: &[Located]) -> bool {
    targets(scope).iter().any(|compound| {
        compound
            .windows(2)
            .any(|pair| pair[0].text == "." && SPINNER_CLASSES.contains(&pair[1].text.as_str()))
    })
}
