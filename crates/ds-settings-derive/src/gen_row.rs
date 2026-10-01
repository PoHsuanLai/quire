//! Codegen for `#[derive(SettingsRow)]` on the struct that is one row of a list-of-tables key:
//! `impl SettingsRow` (a `Column` per field) and `impl ListElement` (so `Vec<Row>` is a
//! `KeyKind::Rows`), design/22-SETTINGS.md section 9.1.

use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

use crate::shape::is_text_type;

/// What one row field's `#[settings(...)]` says.
struct ColumnAttrs {
    label: String,
    pattern: Option<String>,
}

pub(crate) fn expand(input: &syn::DeriveInput) -> syn::Result<TokenStream> {
    let syn::Data::Struct(data) = &input.data else {
        return Err(syn::Error::new(
            input.span(),
            "#[derive(SettingsRow)] only supports a struct with named fields",
        ));
    };
    let syn::Fields::Named(fields) = &data.fields else {
        return Err(syn::Error::new(
            data.fields.span(),
            "#[derive(SettingsRow)] only supports a struct with named fields",
        ));
    };
    let mut columns = Vec::new();
    for field in &fields.named {
        let Some(ident) = &field.ident else {
            continue;
        };
        let attrs = column_attrs(field)?;
        let name = ident.to_string();
        let label = &attrs.label;
        let kind = if is_text_type(&field.ty) {
            let pattern = match &attrs.pattern {
                Some(pattern) => quote! { ::std::option::Option::Some(#pattern.to_owned()) },
                None => quote! { ::std::option::Option::None },
            };
            quote! { ::ds_settings::schema::ColumnKind::Text { pattern: #pattern } }
        } else if attrs.pattern.is_some() {
            return Err(syn::Error::new(
                field.ty.span(),
                "#[settings(pattern = ...)] is for a text column (`String`); a closed enum \
                 column takes its choices from its `Word`",
            ));
        } else {
            let ty = &field.ty;
            quote! { ::ds_settings::schema::choice_of::<#ty>() }
        };
        columns.push(quote! {
            ::ds_settings::schema::Column {
                name: ::ds_settings::schema::ColumnName(#name.to_owned()),
                label: ::ds_settings::schema::Label(#label.to_owned()),
                kind: #kind,
            }
        });
    }
    let ident = &input.ident;
    Ok(quote! {
        impl ::ds_settings::schema::SettingsRow for #ident {
            fn columns() -> ::std::vec::Vec<::ds_settings::schema::Column> {
                ::std::vec![#(#columns),*]
            }
        }

        impl ::ds_settings::schema::ListElement for #ident {
            fn list_kind() -> ::ds_settings::schema::KeyKind {
                ::ds_settings::schema::KeyKind::Rows {
                    columns: <Self as ::ds_settings::schema::SettingsRow>::columns(),
                }
            }
        }
    })
}

fn column_attrs(field: &syn::Field) -> syn::Result<ColumnAttrs> {
    let mut label = None;
    let mut pattern = None;
    for attr in field
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("settings"))
    {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("label") {
                label = Some(meta.value()?.parse::<syn::LitStr>()?.value());
            } else if meta.path.is_ident("pattern") {
                pattern = Some(meta.value()?.parse::<syn::LitStr>()?.value());
            } else {
                return Err(meta.error(
                    "unknown #[settings(...)] attribute on a row field; expected `label` or \
                     `pattern`",
                ));
            }
            Ok(())
        })?;
    }
    let label = label.ok_or_else(|| {
        syn::Error::new(
            field.span(),
            "every field of a #[derive(SettingsRow)] struct needs #[settings(label = \"...\")]",
        )
    })?;
    Ok(ColumnAttrs { label, pattern })
}

#[cfg(test)]
mod tests {
    use super::expand;

    fn expanded(source: &str) -> syn::Result<String> {
        let input: syn::DeriveInput =
            syn::parse_str(source).unwrap_or_else(|e| panic!("{source}: {e}"));
        expand(&input).map(|tokens| tokens.to_string())
    }

    #[test]
    fn a_string_field_is_a_text_column_with_its_pattern() {
        let text =
            expanded("struct R { #[settings(label = \"App\", pattern = \"[a-z]+\")] app: String }")
                .unwrap_or_else(|e| panic!("{e}"));
        assert!(text.contains("ColumnKind :: Text"), "{text}");
        assert!(text.contains("\"[a-z]+\""), "{text}");
    }

    #[test]
    fn any_other_field_is_a_choice_column() {
        let text = expanded("struct R { #[settings(label = \"Side\")] side: Side }")
            .unwrap_or_else(|e| panic!("{e}"));
        assert!(text.contains("choice_of :: < Side >"), "{text}");
    }

    #[test]
    fn a_malformed_row_names_what_is_wrong() {
        const CASES: &[(&str, &str)] = &[
            ("struct R { app: String }", "needs #[settings(label"),
            (
                "struct R { #[settings(label = \"A\", range = \"1..=2\")] app: String }",
                "unknown #[settings(...)] attribute on a row field",
            ),
            (
                "struct R { #[settings(label = \"A\", pattern = \"x\")] side: Side }",
                "pattern = ...",
            ),
            ("struct R(String);", "named fields"),
            ("enum R { A }", "named fields"),
        ];
        for (source, want) in CASES {
            let err = expanded(source).err().map(|e| e.to_string());
            assert!(
                err.as_deref().is_some_and(|e| e.contains(want)),
                "{source}: {err:?}"
            );
        }
    }
}
