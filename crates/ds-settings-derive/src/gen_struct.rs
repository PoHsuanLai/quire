//! Codegen for `#[derive(SettingsSchema)]` on a struct: one `impl SettingsSchema` whose
//! `schema()` builds a `KeySpec` per field from its `#[settings(...)]` attributes and its own
//! type (design/22-SETTINGS.md section 9.1).

use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

use crate::attrs::{FieldAttrs, container_attrs, field_attrs};
use crate::shape::{Shape, shape_of};

pub(crate) fn expand(input: &syn::DeriveInput, data: &syn::DataStruct) -> syn::Result<TokenStream> {
    let syn::Fields::Named(fields) = &data.fields else {
        return Err(syn::Error::new(
            data.fields.span(),
            "#[derive(SettingsSchema)] only supports a struct with named fields",
        ));
    };

    let container = container_attrs(input.span(), &input.attrs)?;
    let app = container
        .file
        .split('/')
        .next()
        .unwrap_or(&container.file)
        .to_owned();
    let file = &container.file;
    let domain = &container.domain;
    let page = &container.page;
    let ident = &input.ident;

    let mut pushes = Vec::new();
    for field in &fields.named {
        let Some(field_ident) = &field.ident else {
            continue;
        };
        let attrs = field_attrs(field.span(), &field.attrs)?;
        let FieldAttrs::Key {
            label,
            help,
            section,
            unit,
            advanced,
            hint,
        } = attrs
        else {
            // `#[settings(skip)]`: a catch-all field such as `extra: toml::Table`, not a key.
            continue;
        };
        let path = format!("{domain}.{field_ident}");
        let exposure = if advanced {
            quote! { ::ds_settings::schema::Exposure::Advanced }
        } else {
            quote! { ::ds_settings::schema::Exposure::Basic }
        };
        let shape = shape_of(&field_ident.to_string(), &field.ty, hint)
            .map_err(|reason| syn::Error::new(field.ty.span(), reason))?;
        let kind = kind_expr(shape, &field.ty, unit.as_deref());
        pushes.push(quote! {
            key.push(::ds_settings::schema::KeySpec {
                path: ::ds_settings::schema::KeyPath(#path.to_owned()),
                kind: #kind,
                default: ::ds_settings::schema::to_value(&default.#field_ident),
                label: ::ds_settings::schema::Label(#label.to_owned()),
                help: ::ds_settings::schema::Help(#help.to_owned()),
                page: #page,
                section: ::ds_settings::schema::Section(#section.to_owned()),
                exposure: #exposure,
                deprecated: ::ds_settings::schema::Deprecated::No,
            });
        });
    }

    Ok(quote! {
        impl ::ds_settings::schema::SettingsSchema for #ident {
            fn schema() -> ::ds_settings::schema::Schema {
                let default = <Self as ::core::default::Default>::default();
                let mut key: ::std::vec::Vec<::ds_settings::schema::KeySpec> = ::std::vec::Vec::new();
                #(#pushes)*
                ::ds_settings::schema::Schema {
                    app: ::ds_settings::schema::AppId(#app.to_owned()),
                    file: ::ds_settings::schema::FilePath(#file.to_owned()),
                    version: 1,
                    key,
                }
            }
        }
    })
}

/// The `KeyKind` expression for one field, from its shape, type and unit.
fn kind_expr(shape: Shape, ty: &syn::Type, unit: Option<&str>) -> TokenStream {
    match shape {
        Shape::Bounded { min, max } => {
            let unit = match unit {
                Some(unit) => quote! { ::std::option::Option::Some(#unit.to_owned()) },
                None => quote! { ::std::option::Option::None },
            };
            quote! { ::ds_settings::schema::KeyKind::Bounded { min: #min, max: #max, unit: #unit } }
        }
        Shape::Text => quote! { ::ds_settings::schema::KeyKind::Text },
        Shape::Colour => quote! { ::ds_settings::schema::KeyKind::Colour },
        Shape::List(inner) => {
            quote! {
                ::ds_settings::schema::KeyKind::List(::std::boxed::Box::new(
                    ::ds_settings::schema::kind_of::<#inner>()
                ))
            }
        }
        Shape::EnumType => quote! { ::ds_settings::schema::kind_of::<#ty>() },
    }
}

#[cfg(test)]
mod tests {
    //! The compile errors a malformed struct gets, checked on the expansion itself: there is
    //! no `trybuild` in this workspace's lockfile to drive a UI test instead.

    use super::expand;

    fn expanded(source: &str) -> syn::Result<proc_macro2::TokenStream> {
        let input: syn::DeriveInput =
            syn::parse_str(source).unwrap_or_else(|e| panic!("{source}: {e}"));
        let syn::Data::Struct(data) = &input.data else {
            panic!("{source}: not a struct");
        };
        expand(&input, data)
    }

    const HEAD: &str = "#[settings(file = \"a/s.toml\", domain = \"d\", page = Page::Dock)]";

    #[test]
    fn a_number_without_a_range_is_missing_range_naming_the_field() {
        const CASES: &[(&str, &str)] = &[
            ("u8", "size"),
            ("Px", "gap_px"),
            ("ds_settings::Ms", "delay_ms"),
        ];
        for (ty, field) in CASES {
            let source = format!("{HEAD} struct S {{ #[settings(label = \"L\")] {field}: {ty} }}");
            let err = expanded(&source).err().map(|e| e.to_string());
            let want = format!("MissingRange {{ field: {field} }}");
            assert!(
                err.as_deref().is_some_and(|e| e.starts_with(&want)),
                "{ty}: {err:?}"
            );
        }
    }

    #[test]
    fn text_fields_are_text_by_type_or_attribute() {
        const CASES: &[&str] = &[
            "#[settings(label = \"L\")] f: String",
            "#[settings(label = \"L\")] f: std::path::PathBuf",
            "#[settings(label = \"L\")] f: Cow<'static, str>",
            "#[settings(label = \"L\", text)] f: SoundTheme",
        ];
        for field in CASES {
            let source = format!("{HEAD} struct S {{ {field} }}");
            let tokens = expanded(&source).unwrap_or_else(|e| panic!("{field}: {e}"));
            let text = tokens.to_string();
            assert!(text.contains("KeyKind :: Text"), "{field}: {text}");
        }
    }

    #[test]
    fn a_number_with_a_range_is_bounded() {
        let source = format!(
            "{HEAD} struct S {{ #[settings(label = \"L\", range = \"1..=9\", unit = \"px\")] f: Px }}"
        );
        let text = expanded(&source)
            .unwrap_or_else(|e| panic!("{e}"))
            .to_string();
        assert!(text.contains("KeyKind :: Bounded"), "{text}");
        assert!(text.contains("min : 1i64"), "{text}");
    }
}
