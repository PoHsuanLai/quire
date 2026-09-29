//! `#[token(...)]` on the enum (`prefix`, `kind`, `css`) and on a variant (`name`, `value`,
//! `light`/`dark`, `calm`/`standard`/`extra`/`reduced`, `system`/`editorial`, `input`).

use syn::{Attribute, LitStr, Path};

/// Whether a consumer writes a token's value inline (`tuned`) or the stylesheet alone owns it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Fixed,
    Tuned,
}

/// What the enum's `#[token(...)]` says.
#[derive(Debug)]
pub(crate) struct EnumTokens {
    pub prefix: String,
    pub kind: Kind,
    /// `css = path`: the function that computes every variant's value.
    pub css: Option<Path>,
}

/// The parsed `#[token(...)]` on the enum.
pub(crate) fn enum_tokens(attrs: &[Attribute]) -> syn::Result<EnumTokens> {
    let mut tokens = EnumTokens {
        prefix: String::new(),
        kind: Kind::Fixed,
        css: None,
    };
    for attr in attrs.iter().filter(|a| a.path().is_ident("token")) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("prefix") {
                tokens.prefix = meta.value()?.parse::<LitStr>()?.value();
            } else if meta.path.is_ident("kind") {
                let word: syn::Ident = meta.value()?.parse()?;
                tokens.kind = match word.to_string().as_str() {
                    "fixed" => Kind::Fixed,
                    "tuned" => Kind::Tuned,
                    _ => return Err(meta.error("kind is `fixed` or `tuned`")),
                };
            } else if meta.path.is_ident("css") {
                tokens.css = Some(meta.value()?.parse::<Path>()?);
            } else {
                return Err(meta.error(
                    "an enum takes `prefix = \"..\"`, `kind = fixed | tuned` or `css = path`",
                ));
            }
            Ok(())
        })?;
    }
    Ok(tokens)
}

/// Which scope a variant's value follows.
#[derive(Debug, Default)]
pub(crate) struct Values {
    pub value: Option<String>,
    pub light: Option<String>,
    pub dark: Option<String>,
    pub calm: Option<String>,
    pub standard: Option<String>,
    pub extra: Option<String>,
    pub reduced: Option<String>,
    pub system: Option<String>,
    pub editorial: Option<String>,
}

/// What the variant's `#[token(...)]` says.
#[derive(Debug, Default)]
pub(crate) struct VariantTokens {
    /// `name = "x"`: the custom property's suffix when it is not the variant's slug.
    pub name: Option<String>,
    /// `input = "--x"`: the property a consumer writes, for a tuned token.
    pub input: Option<String>,
    pub values: Values,
}

/// The parsed `#[token(...)]` on one variant.
pub(crate) fn variant_tokens(attrs: &[Attribute]) -> syn::Result<VariantTokens> {
    let mut tokens = VariantTokens::default();
    for attr in attrs.iter().filter(|a| a.path().is_ident("token")) {
        attr.parse_nested_meta(|meta| {
            let text = meta.value()?.parse::<LitStr>()?.value();
            let slot = |name: &str| meta.path.is_ident(name);
            let values = &mut tokens.values;
            if slot("name") {
                tokens.name = Some(text);
            } else if slot("input") {
                tokens.input = Some(text);
            } else if slot("value") {
                values.value = Some(text);
            } else if slot("light") {
                values.light = Some(text);
            } else if slot("dark") {
                values.dark = Some(text);
            } else if slot("calm") {
                values.calm = Some(text);
            } else if slot("standard") {
                values.standard = Some(text);
            } else if slot("extra") {
                values.extra = Some(text);
            } else if slot("reduced") {
                values.reduced = Some(text);
            } else if slot("system") {
                values.system = Some(text);
            } else if slot("editorial") {
                values.editorial = Some(text);
            } else {
                return Err(meta.error(
                    "a variant takes `name`, `input`, `value`, `light`/`dark`, \
                     `calm`/`standard`/`extra`/`reduced` or `system`/`editorial`",
                ));
            }
            Ok(())
        })?;
    }
    Ok(tokens)
}
