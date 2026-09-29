//! `#[word(...)]` on the enum (`case = snake`) and on a variant (`slug = "x"`, `label = "x"`).

use crate::case::Case;
use syn::{Attribute, LitStr};

/// What the enum's `#[word(...)]` says.
pub(crate) fn enum_case(attrs: &[Attribute]) -> syn::Result<Case> {
    let mut case = Case::Kebab;
    for attr in attrs.iter().filter(|a| a.path().is_ident("word")) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("case") {
                let value: syn::Ident = meta.value()?.parse()?;
                case = match value.to_string().as_str() {
                    "kebab" => Case::Kebab,
                    "snake" => Case::Snake,
                    _ => return Err(meta.error("case is `kebab` or `snake`")),
                };
                Ok(())
            } else {
                Err(meta.error("an enum takes `#[word(case = kebab)]` or `#[word(case = snake)]`"))
            }
        })?;
    }
    Ok(case)
}

/// What a variant's `#[word(...)]` overrides.
#[derive(Debug, Default)]
pub(crate) struct VariantWords {
    /// `slug = "x"`.
    pub slug: Option<String>,
    /// `label = "x"`.
    pub label: Option<String>,
}

/// The parsed `#[word(...)]` on one variant.
pub(crate) fn variant_words(attrs: &[Attribute]) -> syn::Result<VariantWords> {
    let mut words = VariantWords::default();
    for attr in attrs.iter().filter(|a| a.path().is_ident("word")) {
        attr.parse_nested_meta(|meta| {
            let value: LitStr = meta.value()?.parse()?;
            if meta.path.is_ident("slug") {
                words.slug = Some(value.value());
            } else if meta.path.is_ident("label") {
                words.label = Some(value.value());
            } else {
                return Err(meta.error("a variant takes `slug = \"x\"` or `label = \"x\"`"));
            }
            Ok(())
        })?;
    }
    Ok(words)
}
