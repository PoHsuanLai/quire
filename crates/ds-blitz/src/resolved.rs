//! A token's resolved value for Rust geometry: the custom property as the root element's
//! computed style holds it after the cascade (the quire sheet, the look, the person's own
//! sheet), with every `var()` substituted. A dock's gap or a menu's row height that a look may
//! move is read here, not from a settings key.
//!
//! The sheets reach the document as a `<style>` whose text follows the `Styles` signal (a look
//! or a style sheet change restyles the document on the next frame); ds-blitz has no event of
//! its own for it. A caller reads again on the frame after that signal changes and never caches
//! a value across it.

use blitz_dom::{Atom, BaseDocument};
use ds::prelude::Px;

/// Reads of resolved custom properties from a document.
#[derive(Debug)]
pub struct Resolved;

impl Resolved {
    /// `name` (`--dock-gap`) as a length in logical pixels: `Some` only for a plain `<number>px`
    /// value, `None` for a missing property, another unit, or anything not a single length (a
    /// token written as `calc()` is not evaluated, so a geometry token is a plain length).
    pub fn length(doc: &BaseDocument, name: &str) -> Option<Px> {
        let text = Self::text(doc, name)?;
        let number = text.strip_suffix("px")?;
        parse_number(number).map(Px)
    }

    /// `name` (`--ratio`) as a plain `<number>`: `None` for a missing property or a value with
    /// a unit. A duration is not a number here.
    pub fn number(doc: &BaseDocument, name: &str) -> Option<f32> {
        parse_number(&Self::text(doc, name)?)
    }

    /// The substituted text of the property on the root element, trimmed.
    fn text(doc: &BaseDocument, name: &str) -> Option<String> {
        let key = Atom::from(name.strip_prefix("--").filter(|rest| !rest.is_empty())?);
        let styles = doc.try_root_element()?.primary_styles()?;
        let properties = styles.custom_properties();
        let value = properties
            .inherited
            .get(&key)
            .or_else(|| properties.non_inherited.get(&key))?;
        Some(value.as_universal()?.css_text().trim().to_owned())
    }
}

/// A finite decimal with no unit.
fn parse_number(text: &str) -> Option<f32> {
    text.trim()
        .parse::<f32>()
        .ok()
        .filter(|value| value.is_finite())
}
