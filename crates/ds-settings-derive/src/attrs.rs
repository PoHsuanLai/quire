//! `#[settings(...)]` attribute parsing, on the struct (container) and on each field.
//!
//! Kept apart from codegen so the error path — every malformed attribute the derive can be
//! given — is unit-testable without expanding a whole macro (`CONVENTIONS.md#8-tests`: table
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
    /// `footnote(section = "...", text = "...")`, any number of times: the text under that
    /// section of this struct's page.
    pub footnotes: Vec<FootnoteAttr>,
}

/// One `footnote(section = "...", text = "...")` on the struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FootnoteAttr {
    /// The section it goes under.
    pub section: String,
    /// What it says.
    pub text: String,
}

/// `ends = ("Small", "Large")` on a ranged field: the words at the slider's two ends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EndsAttr {
    /// At the low end.
    pub low: String,
    /// At the high end.
    pub high: String,
}

/// What a field's attributes say about its kind, ahead of its type.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// `provided = "look"`: a choice among installed things that provide the capability, stored as
    /// `"standard"` or an id (a `Provided` field).
    Provided {
        /// The capability the choices provide.
        capability: String,
    },
    /// Neither: the type decides (`crate::shape`).
    Infer,
}

/// Whether a field carries `agent_settable`: two states rather than a `bool`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AgentMark {
    /// No mark: only the person sets the key.
    HandsOff,
    /// `agent_settable`: the schema lets an agent set the key.
    Settable,
}

/// `#[settings(label = "...", help = "...", section = "...", range = "a..=b", unit = "...",
/// control = "...", provided = "...", advanced, text, agent_settable)]` on one field, or `#[settings(skip)]` on one that is not a key at all (a
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
        /// `agent_settable` or nothing.
        agent: AgentMark,
        /// `text`, `range` (with its ends), `provided` (with its capability) or none.
        hint: KindHint,
        /// `control = "picture"`: the control that draws the key when its kind's default is not
        /// the one wanted.
        control: Option<String>,
        /// `note = "..."`: the short line shown under the key.
        note: Option<String>,
        /// `ends = ("Small", "Large")`: only with `range`. Boxed: the rare field keeps the enum
        /// small.
        ends: Option<Box<EndsAttr>>,
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
    let mut footnotes = Vec::new();
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
            } else if meta.path.is_ident("footnote") {
                footnotes.push(footnote(&meta)?);
            } else {
                return Err(meta.error(
                    "unknown #[settings(...)] attribute on a struct; expected `file`, \
                     `domain`, `page` or `footnote`",
                ));
            }
            Ok(())
        })?;
    }
    let span = attrs.first().map_or(fallback, Spanned::span);
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
        footnotes,
    })
}

/// `footnote(section = "...", text = "...")`: both are needed and not empty.
fn footnote(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<FootnoteAttr> {
    let mut section = None;
    let mut text = None;
    meta.parse_nested_meta(|inner| {
        if inner.path.is_ident("section") {
            section = Some(lit_str(&inner)?);
        } else if inner.path.is_ident("text") {
            text = Some(lit_str(&inner)?);
        } else {
            return Err(inner.error("a footnote takes `section = \"...\"` and `text = \"...\"`"));
        }
        Ok(())
    })?;
    match (section, text) {
        (Some(section), Some(text)) if !section.is_empty() && !text.is_empty() => {
            Ok(FootnoteAttr { section, text })
        }
        _ => {
            Err(meta.error("a footnote needs a non-empty `section = \"...\"` and `text = \"...\"`"))
        }
    }
}

/// `ends = ("Small", "Large")`: two non-empty strings.
fn ends(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<EndsAttr> {
    let tuple = meta.value()?.parse::<syn::ExprTuple>()?;
    let words: Vec<String> = tuple
        .elems
        .iter()
        .filter_map(|elem| match elem {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(text),
                ..
            }) => Some(text.value()),
            _ => None,
        })
        .collect();
    match (tuple.elems.len(), words.as_slice()) {
        (2, [low, high]) if !low.is_empty() && !high.is_empty() => Ok(EndsAttr {
            low: low.clone(),
            high: high.clone(),
        }),
        _ => Err(meta.error("`ends` takes two words: ends = (\"Small\", \"Large\")")),
    }
}

/// The parsed `#[settings(...)]` on one field's `attrs`.
pub(crate) fn field_attrs(fallback: Span, attrs: &[syn::Attribute]) -> syn::Result<FieldAttrs> {
    let mut label = None;
    let mut help = String::new();
    let mut section = String::new();
    let mut range = None;
    let mut unit = None;
    let mut control = None;
    let mut note = None;
    let mut ends_at = None;
    let mut advanced = false;
    let mut skip = false;
    let mut agent = AgentMark::HandsOff;
    let mut text = false;
    let mut provided = None;
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
            } else if meta.path.is_ident("control") {
                control = Some(lit_str(&meta)?);
            } else if meta.path.is_ident("note") {
                note = Some(lit_str(&meta)?);
            } else if meta.path.is_ident("ends") {
                ends_at = Some(ends(&meta)?);
            } else if meta.path.is_ident("range") {
                let text = lit_str(&meta)?;
                range = Some(parse_range(&text).map_err(|reason| meta.error(reason))?);
            } else if meta.path.is_ident("advanced") {
                advanced = true;
            } else if meta.path.is_ident("agent_settable") {
                agent = AgentMark::Settable;
            } else if meta.path.is_ident("skip") {
                skip = true;
            } else if meta.path.is_ident("text") {
                text = true;
            } else if meta.path.is_ident("provided") {
                provided = Some(lit_str(&meta)?);
            } else {
                return Err(meta.error(
                    "unknown #[settings(...)] attribute on a field; expected `label`, `help`, \
                     `note`, `section`, `range`, `ends`, `unit`, `control`, `provided`, \
                     `advanced`, `text`, `agent_settable` or `skip`",
                ));
            }
            Ok(())
        })?;
    }
    let span = attrs.first().map_or(fallback, Spanned::span);
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
    let hint = match (range, text, provided) {
        (None, false, Some(capability)) if !capability.is_empty() => {
            KindHint::Provided { capability }
        }
        (_, _, Some(_)) => {
            return Err(syn::Error::new(
                span,
                "#[settings(...)]: `provided` names a capability and goes alone, without \
                 `text` or `range`",
            ));
        }
        (Some(_), true, None) => {
            return Err(syn::Error::new(
                span,
                "#[settings(...)]: `text` and `range` cannot both be given; a key is either \
                 free text or a bounded number",
            ));
        }
        (Some((min, max)), false, None) => KindHint::Bounded { min, max },
        (None, true, None) => KindHint::Text,
        (None, false, None) => KindHint::Infer,
    };
    if ends_at.is_some() && !matches!(hint, KindHint::Bounded { .. }) {
        return Err(syn::Error::new(
            span,
            "#[settings(...)]: `ends` labels a slider, so it needs `range`",
        ));
    }
    Ok(FieldAttrs::Key {
        label: label.ok_or_else(|| missing(span, "label"))?,
        help,
        section,
        unit,
        advanced,
        agent,
        hint,
        control,
        note,
        ends: ends_at.map(Box::new),
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
    use super::{AgentMark, FieldAttrs, KindHint, container_attrs, field_attrs, parse_range};
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
    fn agent_settable_marks_a_key_and_its_absence_leaves_it_hands_off() {
        const CASES: &[(&str, AgentMark)] = &[
            (
                "#[settings(label = \"L\", agent_settable)]",
                AgentMark::Settable,
            ),
            ("#[settings(label = \"L\")]", AgentMark::HandsOff),
        ];
        for (attr, want) in CASES {
            let source = format!("struct S {{ {attr} f: u8 }}");
            let attrs = first_field_attrs(&source);
            let parsed = field_attrs(attrs[0].span(), &attrs).unwrap();
            let FieldAttrs::Key { agent, .. } = parsed else {
                panic!("{attr}: expected FieldAttrs::Key");
            };
            assert_eq!(agent, *want, "{attr}");
        }
    }

    #[test]
    fn a_control_is_named_by_its_id_and_absent_otherwise() {
        const CASES: &[(&str, Option<&str>)] = &[
            (
                "#[settings(label = \"L\", control = \"picture\")]",
                Some("picture"),
            ),
            ("#[settings(label = \"L\")]", None),
        ];
        for (attr, want) in CASES {
            let source = format!("struct S {{ {attr} f: String }}");
            let attrs = first_field_attrs(&source);
            let parsed = field_attrs(attrs[0].span(), &attrs).unwrap();
            let FieldAttrs::Key { control, .. } = parsed else {
                panic!("{attr}: expected FieldAttrs::Key");
            };
            assert_eq!(control.as_deref(), *want, "{attr}");
        }
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
    fn provided_names_a_capability_and_goes_alone() {
        let attrs = first_field_attrs(
            "struct S { #[settings(label = \"L\", provided = \"look\")] f: Provided }",
        );
        let FieldAttrs::Key { hint, .. } = field_attrs(attrs[0].span(), &attrs).unwrap() else {
            panic!("expected FieldAttrs::Key");
        };
        assert_eq!(
            hint,
            KindHint::Provided {
                capability: "look".to_owned()
            }
        );
        const BAD: &[&str] = &[
            "#[settings(label = \"L\", provided = \"look\", text)]",
            "#[settings(label = \"L\", provided = \"look\", range = \"0..=9\")]",
            "#[settings(label = \"L\", provided = \"\")]",
        ];
        for attr in BAD {
            let source = format!("struct S {{ {attr} f: Provided }}");
            let attrs = first_field_attrs(&source);
            let err = field_attrs(attrs[0].span(), &attrs).unwrap_err();
            assert!(err.to_string().contains("`provided`"), "{attr}: {err}");
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

    #[test]
    fn notes_ends_and_footnotes_are_checked() {
        // name, field source, the error's words (empty: it parses)
        let fields = [
            (
                "a note",
                "#[settings(label = \"L\", note = \"N\")] f: bool",
                "",
            ),
            (
                "ends on a slider",
                "#[settings(label = \"L\", range = \"1..=9\", ends = (\"Small\", \"Large\"))] f: u8",
                "",
            ),
            (
                "ends without a range",
                "#[settings(label = \"L\", ends = (\"Small\", \"Large\"))] f: bool",
                "needs `range`",
            ),
            (
                "one end",
                "#[settings(label = \"L\", range = \"1..=9\", ends = (\"Small\",))] f: u8",
                "two words",
            ),
            (
                "an empty end",
                "#[settings(label = \"L\", range = \"1..=9\", ends = (\"\", \"Large\"))] f: u8",
                "two words",
            ),
        ];
        for (name, field, want) in fields {
            let attrs = first_field_attrs(&format!("struct S {{ {field} }}"));
            match (field_attrs(attrs[0].span(), &attrs), want) {
                (Ok(_), "") => {}
                (Err(err), want) if !want.is_empty() => {
                    assert!(err.to_string().contains(want), "{name}: {err}");
                }
                (got, _) => panic!("{name}: {:?}", got.map(|_| ())),
            }
        }
        let attrs = struct_attrs(
            "#[settings(file = \"a/b.toml\", domain = \"d\", page = Page::Dock, \
             footnote(section = \"S\", text = \"T\"), footnote(section = \"U\", text = \"V\"))] \
             struct S;",
        );
        let parsed = container_attrs(attrs[0].span(), &attrs).unwrap();
        assert_eq!(parsed.footnotes.len(), 2);
        let attrs = struct_attrs(
            "#[settings(file = \"a/b.toml\", domain = \"d\", page = Page::Dock, \
             footnote(section = \"S\"))] struct S;",
        );
        let err = container_attrs(attrs[0].span(), &attrs).unwrap_err();
        assert!(err.to_string().contains("footnote"), "{err}");
    }
}
