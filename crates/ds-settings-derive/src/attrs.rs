//! `#[settings(...)]` attribute parsing, on the struct (container) and on each field.
//!
//! Kept apart from codegen so the error path — every malformed attribute the derive can be
//! given — is unit-testable without expanding a whole macro (`CONVENTIONS.md#9-tests`: table
//! driven, one case per row). `design/22-SETTINGS.md` section 9.1 is the grammar these parse.

use proc_macro2::Span;
use syn::spanned::Spanned;

/// `#[settings(file = "...", domain = "...", page = ...)]` on the struct.
#[derive(Debug)]
pub(crate) struct ContainerAttrs {
    /// `"quire/appearance.toml"`: the owning app is its first path segment.
    pub file: String,
    /// `"appearance"`: the dotted-path prefix for every field in this struct.
    pub domain: String,
    /// The `Page` variant expression, re-emitted verbatim so it resolves in the caller's scope.
    pub page: syn::Expr,
}

/// What a field's attributes say about its kind, ahead of its type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KindHint {
    /// `range = "a..=b"`: a bounded number from `min` to `max`.
    Bounded {
        /// The low end.
        min: i64,
        /// The high end.
        max: i64,
    },
    /// `text`: free text whatever the type (a `SoundTheme(String)` newtype).
    Text,
    /// Neither: the type decides (`crate::shape`).
    Infer,
}

/// `#[settings(label = "...", help = "...", section = "...", range = "a..=b", unit = "...",
/// advanced, text)]` on one field, or `#[settings(skip)]` on one that is not a key at all (a
/// `#[serde(flatten)] extra: toml::Table` catch-all is the only field this workspace's structs
/// use it for today).
#[derive(Debug)]
pub(crate) enum FieldAttrs {
    /// Not a settings key: emits no `KeySpec`.
    Skip,
    /// A settings key, with its widget-independent metadata.
    Key {
        label: String,
        help: String,
        section: String,
        unit: Option<String>,
        advanced: bool,
        /// `text`, `range` (with its ends) or neither.
        hint: KindHint,
    },
}

/// The parsed `#[settings(...)]` on `attrs`, or the specific reason it is malformed. `fallback`
/// is the span an error points at when no `#[settings(...)]` attribute exists at all to point
/// at instead.
pub(crate) fn container_attrs(
    fallback: Span,
    attrs: &[syn::Attribute],
) -> syn::Result<ContainerAttrs> {
    let mut file = None;
    let mut domain = None;
    let mut page = None;
    let mut found = false;
    for attr in settings_attrs(attrs) {
        found = true;
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("file") {
                file = Some(lit_str(&meta)?);
            } else if meta.path.is_ident("domain") {
                domain = Some(lit_str(&meta)?);
            } else if meta.path.is_ident("page") {
                page = Some(meta.value()?.parse::<syn::Expr>()?);
            } else {
                return Err(meta.error(
                    "unknown #[settings(...)] attribute on a struct; expected `file`, \
                     `domain` or `page`",
                ));
            }
            Ok(())
        })?;
    }
    let span = attrs.first().map(|attr| attr.span()).unwrap_or(fallback);
    if !found {
        return Err(syn::Error::new(
            span,
            "a #[derive(SettingsSchema)] struct needs #[settings(file = \"...\", \
             domain = \"...\", page = ...)]",
        ));
    }
    Ok(ContainerAttrs {
        file: file.ok_or_else(|| missing(span, "file"))?,
        domain: domain.ok_or_else(|| missing(span, "domain"))?,
        page: page.ok_or_else(|| missing(span, "page"))?,
    })
}

/// The parsed `#[settings(...)]` on one field's `attrs`.
pub(crate) fn field_attrs(fallback: Span, attrs: &[syn::Attribute]) -> syn::Result<FieldAttrs> {
    let mut label = None;
    let mut help = String::new();
    let mut section = String::new();
    let mut range = None;
    let mut unit = None;
    let mut advanced = false;
    let mut skip = false;
    let mut text = false;
    let mut found = false;
    for attr in settings_attrs(attrs) {
        found = true;
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("label") {
                label = Some(lit_str(&meta)?);
            } else if meta.path.is_ident("help") {
                help = lit_str(&meta)?;
            } else if meta.path.is_ident("section") {
                section = lit_str(&meta)?;
            } else if meta.path.is_ident("unit") {
                unit = Some(lit_str(&meta)?);
            } else if meta.path.is_ident("range") {
                let text = lit_str(&meta)?;
                range = Some(parse_range(&text).map_err(|reason| meta.error(reason))?);
            } else if meta.path.is_ident("advanced") {
                advanced = true;
            } else if meta.path.is_ident("skip") {
                skip = true;
            } else if meta.path.is_ident("text") {
                text = true;
            } else {
                return Err(meta.error(
                    "unknown #[settings(...)] attribute on a field; expected `label`, `help`, \
                     `section`, `range`, `unit`, `advanced`, `text` or `skip`",
                ));
            }
            Ok(())
        })?;
    }
    let span = attrs.first().map(|attr| attr.span()).unwrap_or(fallback);
    if skip {
        return Ok(FieldAttrs::Skip);
    }
    if !found {
        return Err(syn::Error::new(
            span,
            "every field of a #[derive(SettingsSchema)] struct needs its own \
             #[settings(label = \"...\")] or #[settings(skip)] (design/22-SETTINGS.md section \
             9.5: a key exists only if a KeySpec describes it)",
        ));
    }
    let hint = match (range, text) {
        (Some(_), true) => {
            return Err(syn::Error::new(
                span,
                "#[settings(...)]: `text` and `range` cannot both be given; a key is either \
                 free text or a bounded number",
            ));
        }
        (Some((min, max)), false) => KindHint::Bounded { min, max },
        (None, true) => KindHint::Text,
        (None, false) => KindHint::Infer,
    };
    Ok(FieldAttrs::Key {
        label: label.ok_or_else(|| missing(span, "label"))?,
        help,
        section,
        unit,
        advanced,
        hint,
    })
}

/// `"48..=128"` to `(48, 128)`; anything else names why it could not be read.
pub(crate) fn parse_range(text: &str) -> Result<(i64, i64), String> {
    let (min, max) = text
        .split_once("..=")
        .ok_or_else(|| format!("range {text:?} must be written \"min..=max\""))?;
    let min: i64 = min
        .trim()
        .parse()
        .map_err(|_| format!("range {text:?}: {:?} is not a whole number", min.trim()))?;
    let max: i64 = max
        .trim()
        .parse()
        .map_err(|_| format!("range {text:?}: {:?} is not a whole number", max.trim()))?;
    if min > max {
        return Err(format!("range {text:?}: the low end is above the high end"));
    }
    Ok((min, max))
}

fn settings_attrs(attrs: &[syn::Attribute]) -> impl Iterator<Item = &syn::Attribute> {
    attrs.iter().filter(|attr| attr.path().is_ident("settings"))
}

fn lit_str(meta: &syn::meta::ParseNestedMeta) -> syn::Result<String> {
    Ok(meta.value()?.parse::<syn::LitStr>()?.value())
}

fn missing(span: Span, key: &str) -> syn::Error {
    syn::Error::new(span, format!("#[settings(...)]: `{key}` is required"))
}

#[cfg(test)]
mod tests {
    use super::{FieldAttrs, KindHint, container_attrs, field_attrs, parse_range};
    use proc_macro2::Span;
    use syn::spanned::Spanned;

    fn struct_attrs(source: &str) -> Vec<syn::Attribute> {
        syn::parse_str::<syn::ItemStruct>(source)
            .unwrap_or_else(|e| panic!("{source}: {e}"))
            .attrs
    }

    fn first_field_attrs(source: &str) -> Vec<syn::Attribute> {
        let item =
            syn::parse_str::<syn::ItemStruct>(source).unwrap_or_else(|e| panic!("{source}: {e}"));
        item.fields
            .iter()
            .next()
            .unwrap_or_else(|| panic!("{source}: no fields"))
            .attrs
            .clone()
    }

    #[test]
    fn a_well_formed_container_parses() {
        let attrs = struct_attrs(
            "#[settings(file = \"sill/settings.toml\", domain = \"dock\", page = Page::Dock)]\
             struct S;",
        );
        let parsed = container_attrs(attrs[0].span(), &attrs).unwrap();
        assert_eq!(parsed.file, "sill/settings.toml");
        assert_eq!(parsed.domain, "dock");
    }

    #[test]
    fn a_container_with_no_settings_attribute_is_an_error() {
        let attrs = struct_attrs("struct S;");
        let err = container_attrs(Span::call_site(), &attrs).unwrap_err();
        assert!(err.to_string().contains("needs #[settings"), "{err}");
    }

    #[test]
    fn a_container_missing_file_is_an_error() {
        let attrs = struct_attrs("#[settings(domain = \"dock\", page = Page::Dock)] struct S;");
        let err = container_attrs(attrs[0].span(), &attrs).unwrap_err();
        assert!(err.to_string().contains("`file`"), "{err}");
    }

    #[test]
    fn a_container_missing_domain_is_an_error() {
        let attrs = struct_attrs("#[settings(file = \"a/b.toml\", page = Page::Dock)] struct S;");
        let err = container_attrs(attrs[0].span(), &attrs).unwrap_err();
        assert!(err.to_string().contains("`domain`"), "{err}");
    }

    #[test]
    fn a_container_missing_page_is_an_error() {
        let attrs = struct_attrs("#[settings(file = \"a/b.toml\", domain = \"dock\")] struct S;");
        let err = container_attrs(attrs[0].span(), &attrs).unwrap_err();
        assert!(err.to_string().contains("`page`"), "{err}");
    }

    #[test]
    fn an_unknown_container_key_is_an_error() {
        let attrs = struct_attrs(
            "#[settings(file = \"a/b.toml\", domain = \"dock\", page = Page::Dock, oops = 1)]\
             struct S;",
        );
        let err = container_attrs(attrs[0].span(), &attrs).unwrap_err();
        assert!(err.to_string().contains("unknown"), "{err}");
    }

    #[test]
    fn a_well_formed_field_parses() {
        let attrs = first_field_attrs(
            "struct S { #[settings(label = \"Magnified size\", range = \"48..=128\", \
             unit = \"px\", section = \"Magnification\", advanced)] f: u8 }",
        );
        let parsed = field_attrs(attrs[0].span(), &attrs).unwrap();
        let FieldAttrs::Key {
            label,
            hint,
            unit,
            section,
            advanced,
            ..
        } = parsed
        else {
            panic!("expected FieldAttrs::Key");
        };
        assert_eq!(label, "Magnified size");
        assert_eq!(hint, KindHint::Bounded { min: 48, max: 128 });
        assert_eq!(unit.as_deref(), Some("px"));
        assert_eq!(section, "Magnification");
        assert!(advanced);
    }

    #[test]
    fn the_hint_follows_text_and_range() {
        const CASES: &[(&str, KindHint)] = &[
            ("#[settings(label = \"L\", text)]", KindHint::Text),
            (
                "#[settings(label = \"L\", range = \"0..=9\")]",
                KindHint::Bounded { min: 0, max: 9 },
            ),
            ("#[settings(label = \"L\")]", KindHint::Infer),
        ];
        for (attr, want) in CASES {
            let source = format!("struct S {{ {attr} f: u8 }}");
            let attrs = first_field_attrs(&source);
            let parsed = field_attrs(attrs[0].span(), &attrs).unwrap();
            let FieldAttrs::Key { hint, .. } = parsed else {
                panic!("{attr}: expected FieldAttrs::Key");
            };
            assert_eq!(hint, *want, "{attr}");
        }
    }

    #[test]
    fn text_and_range_together_are_an_error() {
        let attrs = first_field_attrs(
            "struct S { #[settings(label = \"L\", text, range = \"0..=9\")] f: u8 }",
        );
        let err = field_attrs(attrs[0].span(), &attrs).unwrap_err();
        assert!(err.to_string().contains("`text` and `range`"), "{err}");
    }

    #[test]
    fn a_skipped_field_needs_no_label() {
        let attrs = first_field_attrs("struct S { #[settings(skip)] f: toml::Table }");
        let parsed = field_attrs(attrs[0].span(), &attrs).unwrap();
        assert!(matches!(parsed, FieldAttrs::Skip));
    }

    #[test]
    fn a_field_with_no_settings_attribute_is_an_error() {
        let attrs = first_field_attrs("struct S { f: u8 }");
        let err = field_attrs(Span::call_site(), &attrs).unwrap_err();
        assert!(err.to_string().contains("needs its own"), "{err}");
    }

    #[test]
    fn a_field_missing_label_is_an_error() {
        let attrs = first_field_attrs("struct S { #[settings(help = \"x\")] f: u8 }");
        let err = field_attrs(attrs[0].span(), &attrs).unwrap_err();
        assert!(err.to_string().contains("`label`"), "{err}");
    }

    #[test]
    fn an_unknown_field_key_is_an_error() {
        let attrs = first_field_attrs("struct S { #[settings(label = \"L\", oops = 1)] f: u8 }");
        let err = field_attrs(attrs[0].span(), &attrs).unwrap_err();
        assert!(err.to_string().contains("unknown"), "{err}");
    }

    #[test]
    fn range_reads_an_inclusive_span() {
        const OK: &[(&str, (i64, i64))] = &[("0..=100", (0, 100)), ("-5..=5", (-5, 5))];
        for (text, want) in OK {
            assert_eq!(parse_range(text), Ok(*want), "{text}");
        }
    }

    #[test]
    fn range_rejects_malformed_text() {
        const BAD: &[&str] = &["48..128", "a..=128", "48..=b", "100..=0", "nope"];
        for text in BAD {
            assert!(parse_range(text).is_err(), "{text} should not parse");
        }
    }
}
