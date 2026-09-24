//! A `style` attribute's declarations, checked one by one: a literal colour or duration is an
//! offence, except inside a custom property a quire component writes on its own element.
//!
//! Some values quire can only compute per instance and must write inline: the Space's `--f-*`
//! frame variables and the layer gradient `--f-grad` (design/03-COLOR.md section 4), an
//! avatar's `--av-bg`, a provider mark's `--pc`. They are custom properties on an element that
//! carries `ds` or a `ds-*` class, and the component's stylesheet reads them through `var()`. The same
//! literal in a non-custom property (`background:#fff`) stays an offence wherever it is: that
//! is a paint written past the stylesheet, not a value handed to it.

use super::colours;
use super::declaration::COLOUR_FUNCTIONS;
use super::kind;
use super::rule::Rule;
use super::tokenize::{self, Located};

/// Who drew the element whose `style` this is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Owner {
    /// It carries the root's `ds` class or a `ds-*` one: a quire component wrote it.
    Quire,
    /// Anything else.
    Consumer,
}

/// Inline colours and durations in a `style` attribute, as `(rule, line, column, text)`, with
/// `base_line`/`base_column` the value's first character in the document.
pub(super) fn offences(
    value: &str,
    owner: Owner,
    base_line: u32,
    base_column: u32,
) -> Vec<(Rule, u32, u32, String)> {
    let tokens: Vec<Located> = tokenize::tokens(value)
        .into_iter()
        .filter(|token| !kind::is_trivial(&token.text))
        .collect();
    tokens
        .split(|token| token.text == ";")
        .filter(|declaration| !(owner == Owner::Quire && is_custom_property(declaration)))
        .flat_map(|declaration| declaration.iter().filter_map(offence))
        .map(|(rule, token)| {
            let (line, column) = if token.line == 1 {
                (base_line, base_column + token.column - 1)
            } else {
                (base_line + token.line - 1, token.column)
            };
            (rule, line, column, format!("style=\"...\": {}", token.text))
        })
        .collect()
}

/// Whether a declaration's property (its first token) is a custom property, `--*`.
fn is_custom_property(declaration: &[Located]) -> bool {
    declaration
        .first()
        .is_some_and(|property| property.text.starts_with("--"))
}

/// The rule one token of an inline declaration breaks, if any.
fn offence(token: &Located) -> Option<(Rule, &Located)> {
    let text = &token.text;
    let rule = if kind::is_hash(text) {
        Some(Rule::HexColour)
    } else if let Some(name) = kind::function_name(text) {
        COLOUR_FUNCTIONS
            .contains(&name.to_ascii_lowercase().as_str())
            .then_some(Rule::ColourFunction)
    } else if kind::is_ident(text) && !text.eq_ignore_ascii_case("currentcolor") {
        colours::is_named_colour(text).then_some(Rule::NamedColour)
    } else {
        kind::dimension_unit(text).and_then(|unit| {
            (unit.eq_ignore_ascii_case("ms") || unit.eq_ignore_ascii_case("s"))
                .then_some(Rule::RawDuration)
        })
    };
    rule.map(|rule| (rule, token))
}
