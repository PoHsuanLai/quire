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
            range,
            unit,
            advanced,
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
        let kind = kind_expr(&field.ty, range, unit.as_deref())?;
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

/// The `KeyKind` expression for one field, from its type and its `#[settings(...)]`.
fn kind_expr(
    ty: &syn::Type,
    range: Option<(i64, i64)>,
    unit: Option<&str>,
) -> syn::Result<TokenStream> {
    Ok(match shape_of(ty, range.is_some()) {
        Shape::Bounded => {
            let (min, max) = range.expect("Shape::Bounded only returns when range is Some");
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
    })
}
